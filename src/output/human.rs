use crate::engine::pipeline::PipelineResult;
use crate::inference::types::ProviderDecision;

pub fn render_human(result: &PipelineResult) -> String {
    let mut out = String::new();
    out.push_str("jev-ops analysis\n\n");

    out.push_str(&format!(
        "{:<12}{:<14} {}\n",
        "Pack:",
        format!("{} {}", result.pack.name, result.pack.version),
        ""
    ));
    out.push_str(&format!("{:<12}{:<14}\n", "Provider:", result.provider));
    out.push_str(&format!(
        "{:<12}{:<14}\n\n",
        "Input:",
        format_bytes(result.input.bytes)
    ));

    for (name, decision) in &result.decisions {
        let label = format_label(name);
        let val_str = match decision {
            ProviderDecision::Choice { value, .. } => value.clone(),
            ProviderDecision::Score { value, .. } => {
                if name.contains("severity") {
                    format!("{}/5", value)
                } else {
                    value.to_string()
                }
            }
            ProviderDecision::Boolean { value, .. } => {
                if *value {
                    "yes".to_string()
                } else {
                    "no".to_string()
                }
            }
        };

        let conf_pct = format!("{:.0}%", decision.confidence() * 100.0);
        let mut line = format!("{:<12}{:<14} {:>4}", label, val_str, conf_pct);

        if let Some(threshold) = result.min_confidence {
            if decision.confidence() < threshold {
                line.push_str("  [LOW CONFIDENCE]");
            }
        }

        out.push_str(&line);
        out.push('\n');
    }

    out
}

fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn format_label(name: &str) -> String {
    let clean = if name == "needs_attention" {
        "Attention".to_string()
    } else {
        let mut chars = name.replace('_', " ").chars().collect::<Vec<_>>();
        if let Some(first) = chars.first_mut() {
            *first = first.to_ascii_uppercase();
        }
        chars.into_iter().collect()
    };
    format!("{}:", clean)
}
