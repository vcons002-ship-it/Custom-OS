//! The local/escalation router (docs/06-hybrid-ai.md §2.2). A pure policy
//! engine: given how confident the on-device tier is, how heavy the task is,
//! how sensitive the content is, connectivity, and the owner's Privacy Dial,
//! it decides whether to answer locally or escalate — and always explains why.
//!
//! The `reason` is user-visible: a legible router is what separates "hybrid"
//! from "leaky". Escalation targets (owner's Ollama / Gemini) live in `gated`;
//! this module only decides *whether* to leave the device.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Engine {
    Local,
    Escalate,
}

/// The owner's Privacy Dial position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Dial {
    Airgapped,
    Balanced,
    CloudBoosted,
}

/// Signals the router weighs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RouteInput {
    /// How sure the local tier is of its own first-pass answer (0..1).
    pub local_confidence: f32,
    /// Task heaviness: multi-step / generative / cross-content (0..1).
    pub complexity: f32,
    /// Content flagged sensitive/private by the redaction gate.
    pub sensitive: bool,
    /// Network reachable.
    pub online: bool,
    pub dial: Dial,
}

impl Default for RouteInput {
    fn default() -> Self {
        Self {
            local_confidence: 1.0,
            complexity: 0.0,
            sensitive: false,
            online: true,
            dial: Dial::Balanced,
        }
    }
}

/// The router's decision, with its reason and whether egress must be redacted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteDecision {
    pub engine: Engine,
    /// Confidence in the routing choice itself (0..1).
    pub confidence: f32,
    /// User-visible explanation.
    pub reason: String,
    /// If escalating: redact sensitive spans before the request leaves.
    pub redact: bool,
}

const CONF_FLOOR: f32 = 0.6;
const COMPLEXITY_CEIL: f32 = 0.6;

/// Decide where a task runs.
pub fn route(inp: RouteInput) -> RouteDecision {
    // Hard local: no network, or the owner airgapped the machine.
    if !inp.online {
        return local(0.99, "offline — local only; cloud-class work queues");
    }
    if inp.dial == Dial::Airgapped {
        return local(0.99, "privacy dial: airgapped — nothing leaves the device");
    }

    let low_conf = inp.local_confidence < CONF_FLOOR;
    let heavy = inp.complexity > COMPLEXITY_CEIL;
    if !low_conf && !heavy {
        return local(
            0.8,
            "local is confident and the task is light — no need to escalate",
        );
    }

    // Escalation is warranted. Assemble the reason.
    let mut why = Vec::new();
    if low_conf {
        why.push(format!(
            "local confidence {:.2} < {CONF_FLOOR}",
            inp.local_confidence
        ));
    }
    if heavy {
        why.push(format!(
            "complexity {:.2} > {COMPLEXITY_CEIL}",
            inp.complexity
        ));
    }
    let drivers = why.join("; ");

    match inp.dial {
        Dial::CloudBoosted => RouteDecision {
            engine: Engine::Escalate,
            confidence: 0.85,
            reason: format!("escalating ({drivers}); dial cloud-boosted"),
            redact: inp.sensitive,
        },
        // Balanced: escalate, but sensitive content is redacted/held first.
        Dial::Balanced => {
            RouteDecision {
                engine: Engine::Escalate,
                confidence: if inp.sensitive { 0.7 } else { 0.85 },
                reason: if inp.sensitive {
                    format!("escalating with redaction ({drivers}); sensitive content held for the gate")
                } else {
                    format!("escalating ({drivers})")
                },
                redact: inp.sensitive,
            }
        }
        Dial::Airgapped => unreachable!("airgapped handled above"),
    }
}

fn local(confidence: f32, reason: &str) -> RouteDecision {
    RouteDecision {
        engine: Engine::Local,
        confidence,
        reason: reason.into(),
        redact: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confident_light_task_stays_local() {
        let d = route(RouteInput {
            local_confidence: 0.9,
            complexity: 0.2,
            ..Default::default()
        });
        assert_eq!(d.engine, Engine::Local);
        assert!(!d.redact);
    }

    #[test]
    fn low_confidence_escalates() {
        let d = route(RouteInput {
            local_confidence: 0.3,
            ..Default::default()
        });
        assert_eq!(d.engine, Engine::Escalate);
        assert!(d.reason.contains("local confidence"));
    }

    #[test]
    fn heavy_task_escalates() {
        let d = route(RouteInput {
            complexity: 0.9,
            ..Default::default()
        });
        assert_eq!(d.engine, Engine::Escalate);
        assert!(d.reason.contains("complexity"));
    }

    #[test]
    fn offline_forces_local_even_if_heavy() {
        let d = route(RouteInput {
            complexity: 0.9,
            online: false,
            ..Default::default()
        });
        assert_eq!(d.engine, Engine::Local);
        assert!(d.reason.contains("offline"));
    }

    #[test]
    fn airgapped_never_escalates() {
        let d = route(RouteInput {
            local_confidence: 0.1,
            complexity: 1.0,
            dial: Dial::Airgapped,
            ..Default::default()
        });
        assert_eq!(d.engine, Engine::Local);
    }

    #[test]
    fn balanced_sensitive_escalation_redacts() {
        let d = route(RouteInput {
            local_confidence: 0.3,
            sensitive: true,
            dial: Dial::Balanced,
            ..Default::default()
        });
        assert_eq!(d.engine, Engine::Escalate);
        assert!(d.redact, "sensitive content must be redacted before egress");
        assert!(d.reason.contains("redaction"));
    }

    #[test]
    fn reason_is_never_empty() {
        for lc in [0.1_f32, 0.9] {
            for cx in [0.1_f32, 0.9] {
                for sensitive in [false, true] {
                    for dial in [Dial::Balanced, Dial::CloudBoosted, Dial::Airgapped] {
                        let d = route(RouteInput {
                            local_confidence: lc,
                            complexity: cx,
                            sensitive,
                            online: true,
                            dial,
                        });
                        assert!(!d.reason.is_empty());
                    }
                }
            }
        }
    }
}
