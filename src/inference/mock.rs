use std::collections::BTreeMap;

use crate::error::Result;
use crate::inference::provider::InferenceProvider;
use crate::inference::types::{InferenceRequest, InferenceResponse, ProviderDecision};
use crate::packs::manifest::DecisionSpec;

pub struct MockInferenceProvider;

impl MockInferenceProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockInferenceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceProvider for MockInferenceProvider {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn infer(&self, request: &InferenceRequest) -> Result<InferenceResponse> {
        let text = &request.input_text;
        let mut decisions = BTreeMap::new();

        // Detect scenario based on fixture markers
        let is_ext4 = text.contains("EXT4-fs error")
            || text.contains("ext4_lookup")
            || text.contains("deleted inode referenced");

        let is_oom = text.contains("Out of memory")
            || text.contains("Killed process")
            || text.contains("oom-kill")
            || text.contains("invoked oom-killer");

        let is_k8s_crashloop = text.contains("CrashLoopBackOff")
            || text.contains("Back-off restarting failed container");

        let is_k8s_oom = text.contains("OOMKilled");

        let is_ssh_brute = text.contains("Failed password")
            || text.contains("Invalid user")
            || (text.contains("sshd") && text.contains("authentication failure"));

        let is_healthy = text.contains("Active: active (running)")
            || text.contains("Started Service")
            || text.contains("All systems operational")
            || text.contains("status: Running");

        let is_aws = text.contains("OutOfMemoryError")
            || text.contains("ECS Agent")
            || text.contains("ExitCode: 137")
            || text.contains("/aws/ecs")
            || text.contains("awslogs");

        // Match specific phrases only: bare substrings like "502" or "aks" hit PIDs and words like "leaks".
        let is_azure = text.contains("App Gateway")
            || text.contains("Application Gateway")
            || text.contains("502 Bad Gateway")
            || text.contains("Health probe")
            || text.contains("AKS")
            || text.contains("Alert Rule");

        let is_gcp = text.contains("Cloud Run")
            || text.contains("Memory limit of")
            || text.contains("exceeded with")
            || text.contains("cloud_run");

        for (name, spec) in &request.decisions {
            let decision = match spec {
                DecisionSpec::Choice { values } => {
                    let (chosen_val, conf) = if is_ext4 {
                        if name == "category" && values.contains(&"filesystem".to_string()) {
                            ("filesystem".to_string(), 0.98)
                        } else if name == "health" && values.contains(&"unhealthy".to_string()) {
                            ("unhealthy".to_string(), 0.96)
                        } else {
                            select_fallback_choice(values)
                        }
                    } else if is_oom {
                        if name == "category" && values.contains(&"memory".to_string()) {
                            ("memory".to_string(), 0.97)
                        } else if name == "health" && values.contains(&"unhealthy".to_string()) {
                            ("unhealthy".to_string(), 0.96)
                        } else if name == "root_cause"
                            && (values.contains(&"resources".to_string())
                                || values.contains(&"memory".to_string()))
                        {
                            let v = if values.contains(&"memory".to_string()) {
                                "memory".to_string()
                            } else {
                                "resources".to_string()
                            };
                            (v, 0.97)
                        } else {
                            select_fallback_choice(values)
                        }
                    } else if is_k8s_crashloop {
                        if name == "root_cause" && values.contains(&"crashloop".to_string()) {
                            ("crashloop".to_string(), 0.96)
                        } else if name == "health" && values.contains(&"unhealthy".to_string()) {
                            ("unhealthy".to_string(), 0.95)
                        } else {
                            select_fallback_choice(values)
                        }
                    } else if is_k8s_oom {
                        if name == "root_cause" && values.contains(&"resources".to_string()) {
                            ("resources".to_string(), 0.98)
                        } else if name == "root_cause" && values.contains(&"oom".to_string()) {
                            ("oom".to_string(), 0.98)
                        } else if name == "health" && values.contains(&"unhealthy".to_string()) {
                            ("unhealthy".to_string(), 0.97)
                        } else {
                            select_fallback_choice(values)
                        }
                    } else if is_ssh_brute {
                        if name == "category" && values.contains(&"security".to_string()) {
                            ("security".to_string(), 0.95)
                        } else if name == "health" && values.contains(&"degraded".to_string()) {
                            ("degraded".to_string(), 0.89)
                        } else {
                            select_fallback_choice(values)
                        }
                    } else if is_aws {
                        if name == "health" && values.contains(&"unhealthy".to_string()) {
                            ("unhealthy".to_string(), 0.96)
                        } else if name == "service" && values.contains(&"ecs".to_string()) {
                            ("ecs".to_string(), 0.98)
                        } else if name == "root_cause" && values.contains(&"oom_killed".to_string())
                        {
                            ("oom_killed".to_string(), 0.97)
                        } else if name == "recommended_action"
                            && values.contains(&"ssm_restart".to_string())
                        {
                            ("ssm_restart".to_string(), 0.94)
                        } else {
                            select_fallback_choice(values)
                        }
                    } else if is_azure {
                        if name == "health" && values.contains(&"degraded".to_string()) {
                            ("degraded".to_string(), 0.91)
                        } else if name == "resource_type"
                            && values.contains(&"app_gateway".to_string())
                        {
                            ("app_gateway".to_string(), 0.95)
                        } else if name == "root_cause"
                            && values.contains(&"probe_failure".to_string())
                        {
                            ("probe_failure".to_string(), 0.93)
                        } else if name == "recommended_action"
                            && values.contains(&"auto_heal".to_string())
                        {
                            ("auto_heal".to_string(), 0.89)
                        } else {
                            select_fallback_choice(values)
                        }
                    } else if is_gcp {
                        if name == "health" && values.contains(&"unhealthy".to_string()) {
                            ("unhealthy".to_string(), 0.96)
                        } else if name == "service" && values.contains(&"cloud_run".to_string()) {
                            ("cloud_run".to_string(), 0.97)
                        } else if name == "root_cause"
                            && values.contains(&"container_exited_137".to_string())
                        {
                            ("container_exited_137".to_string(), 0.96)
                        } else if name == "recommended_action"
                            && values.contains(&"restart_revision".to_string())
                        {
                            ("restart_revision".to_string(), 0.92)
                        } else {
                            select_fallback_choice(values)
                        }
                    } else if is_healthy {
                        if name == "health" && values.contains(&"healthy".to_string()) {
                            ("healthy".to_string(), 0.98)
                        } else if name == "category" && values.contains(&"normal".to_string()) {
                            ("normal".to_string(), 0.95)
                        } else {
                            select_fallback_choice(values)
                        }
                    } else {
                        select_fallback_choice(values)
                    };

                    ProviderDecision::Choice {
                        value: chosen_val,
                        confidence: conf,
                        probabilities: None,
                    }
                }
                DecisionSpec::Score { min, max, .. } => {
                    let (val, conf) = if is_ext4 {
                        (*max, 0.91)
                    } else if is_oom || is_k8s_oom || is_aws || is_gcp {
                        (4.max(*min).min(*max), 0.94)
                    } else if is_k8s_crashloop || is_azure {
                        (4.max(*min).min(*max), 0.92)
                    } else if is_ssh_brute {
                        (3.max(*min).min(*max), 0.88)
                    } else if is_healthy {
                        (*min, 0.95)
                    } else {
                        (*min, 0.75)
                    };

                    ProviderDecision::Score {
                        value: val,
                        confidence: conf,
                        probabilities: None,
                    }
                }
                DecisionSpec::Boolean => {
                    let (val, conf) = if is_ext4 {
                        (true, 0.99)
                    } else if is_oom
                        || is_k8s_oom
                        || is_k8s_crashloop
                        || is_aws
                        || is_azure
                        || is_gcp
                    {
                        (true, 0.98)
                    } else if is_ssh_brute {
                        (true, 0.92)
                    } else if is_healthy {
                        (false, 0.99)
                    } else {
                        (false, 0.80)
                    };

                    ProviderDecision::Boolean {
                        value: val,
                        confidence: conf,
                    }
                }
            };

            decisions.insert(name.clone(), decision);
        }

        Ok(InferenceResponse {
            provider: self.name().to_string(),
            decisions,
        })
    }
}

fn select_fallback_choice(values: &[String]) -> (String, f64) {
    if values.contains(&"unknown".to_string()) {
        ("unknown".to_string(), 0.70)
    } else if !values.is_empty() {
        (values[0].clone(), 0.70)
    } else {
        ("unknown".to_string(), 0.70)
    }
}
