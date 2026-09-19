use assert_cmd::Command;
use predicates::prelude::*;

use jev_ops::inference::resolve_provider;

#[test]
fn test_resolve_mock_provider() {
    let provider = resolve_provider(Some("mock"), None).unwrap();
    assert_eq!(provider.name(), "mock");
}

#[test]
fn test_resolve_typesafe_provider_with_explicit_key() {
    let provider = resolve_provider(Some("typesafe"), Some("ts_test_12345")).unwrap();
    assert_eq!(provider.name(), "typesafe-jev");
}

#[test]
fn test_resolve_typesafe_provider_missing_key() {
    // Ensure environment does not leak a key during this test
    std::env::remove_var("TYPESAFE_API_KEY");
    let res = resolve_provider(Some("typesafe"), None);
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    assert!(err.contains("Missing API key"));
    assert!(err.contains("TYPESAFE_API_KEY"));
}

#[test]
fn test_resolve_unknown_provider() {
    let res = resolve_provider(Some("non-existent-provider"), None);
    assert!(res.is_err());
    let err = res.err().unwrap().to_string();
    assert!(err.contains("Unknown inference provider"));
}

#[test]
fn test_cli_typesafe_missing_api_key_exit_code() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.env_remove("TYPESAFE_API_KEY");
    cmd.args([
        "analyze",
        "linux",
        "--provider",
        "typesafe",
        "--input",
        "tests/fixtures/linux/healthy.txt",
    ]);
    cmd.assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("Missing API key"));
}

#[test]
fn test_cli_no_provider_and_no_key_fails_instead_of_mocking() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.env_remove("TYPESAFE_API_KEY");
    cmd.args([
        "analyze",
        "linux",
        "--input",
        "tests/fixtures/linux/healthy.txt",
    ]);
    cmd.assert()
        .failure()
        .code(5)
        .stderr(predicate::str::contains("--provider mock"));
}

#[test]
fn test_cli_explicit_mock_provider() {
    let mut cmd = Command::cargo_bin("jev-ops").unwrap();
    cmd.env_remove("TYPESAFE_API_KEY");
    cmd.args([
        "analyze",
        "linux",
        "--provider",
        "mock",
        "--input",
        "tests/fixtures/linux/ext4-error.txt",
        "--json",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"provider\": \"mock\""))
        .stdout(predicate::str::contains("\"filesystem\""));
}

mod http_stub {
    use std::collections::BTreeMap;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    use jev_ops::inference::{InferenceProvider, InferenceRequest, TypeSafeJevProvider};
    use jev_ops::packs::manifest::DecisionSpec;

    /// Serves one request with `response_body`; returns the base URL and the captured request body.
    fn serve_once(response_body: &'static str) -> (String, mpsc::Receiver<serde_json::Value>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/v1", listener.local_addr().unwrap());
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut content_length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    content_length = v.trim().parse().unwrap();
                }
            }
            let mut body = vec![0; content_length];
            reader.read_exact(&mut body).unwrap();
            tx.send(serde_json::from_slice(&body).unwrap()).unwrap();
            let mut stream = stream;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            )
            .unwrap();
        });
        (url, rx)
    }

    fn request(decisions: BTreeMap<String, DecisionSpec>) -> InferenceRequest {
        InferenceRequest {
            pack_name: "test".to_string(),
            pack_version: "0.1.0".to_string(),
            instructions: "PACK GUIDANCE: 1 = cosmetic, 5 = total outage".to_string(),
            decisions,
            input_text: "boom: total outage".to_string(),
        }
    }

    fn score_spec(min: i64, max: i64, levels: &[&str]) -> BTreeMap<String, DecisionSpec> {
        let mut d = BTreeMap::new();
        d.insert(
            "severity".to_string(),
            DecisionSpec::Score {
                min,
                max,
                levels: levels.iter().map(|s| s.to_string()).collect(),
            },
        );
        d
    }

    fn provider(url: String) -> TypeSafeJevProvider {
        TypeSafeJevProvider::with_custom("ts_test".to_string(), Some(url), None)
    }

    #[test]
    fn score_level_index_maps_to_pack_range_and_guidance_is_sent() {
        let (url, rx) = serve_once(
            r#"{"model":"jev-latest","answers":{"severity":{"type":"score","score":4.0,"confidence":0.9,
                "legend":{"0":"a","1":"b","2":"c","3":"d","4":"e"},
                "probabilities":{"0":0.0,"1":0.0,"2":0.0,"3":0.1,"4":0.9}}},
                "usage":{"input_tokens":1,"output_tokens":1}}"#,
        );
        let levels = ["cosmetic", "minor", "degraded", "major", "total outage"];
        let res = provider(url)
            .infer(&request(score_spec(1, 5, &levels)))
            .unwrap();

        match &res.decisions["severity"] {
            jev_ops::inference::ProviderDecision::Score {
                value,
                probabilities,
                ..
            } => {
                assert_eq!(*value, 5, "top level (index 4) must map to pack max 5");
                let probs = probabilities.as_ref().unwrap();
                assert_eq!(probs["5"], 0.9);
                assert!(!probs.contains_key("0"));
            }
            other => panic!("expected score, got {:?}", other),
        }

        let sent = rx.recv().unwrap();
        assert_eq!(
            sent["state"]["analysis_guidance"],
            "PACK GUIDANCE: 1 = cosmetic, 5 = total outage"
        );
        assert_eq!(sent["state"]["diagnostic_input"], "boom: total outage");
        assert_eq!(sent["questions"]["severity"]["criteria"][4], "total outage");
    }

    #[test]
    fn score_outside_requested_levels_is_rejected_not_clamped() {
        let (url, _rx) =
            serve_once(r#"{"answers":{"severity":{"type":"score","score":7.0,"confidence":0.9}}}"#);
        let err = provider(url)
            .infer(&request(score_spec(0, 5, &[])))
            .unwrap_err();
        assert_eq!(err.exit_code(), 6);
        assert!(err.to_string().contains("outside the requested levels"));
    }

    #[test]
    fn noul_outside_unit_range_is_rejected() {
        let (url, _rx) = serve_once(r#"{"answers":{"page":{"type":"noul","noul":1.7}}}"#);
        let mut d = BTreeMap::new();
        d.insert("page".to_string(), DecisionSpec::Boolean);
        let err = provider(url).infer(&request(d)).unwrap_err();
        assert_eq!(err.exit_code(), 6);
    }
}
