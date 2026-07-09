//! modeld — the on-device model runtime (docs/06-hybrid-ai.md).
//!
//! No args (or "daemon") → the supervised service weaved launches (announce +
//! heartbeat for now; the model host lands when weights ship on the owner's
//! GPU box). Subcommands expose the model-free pieces already usable today:
//!   modeld facets <path...>   — Facet Resolution (deterministic tier)
//!   modeld route [flags]      — explain a local-vs-escalate routing decision
//!
//! The CLIP/LLM tiers plug in behind `clade_facets::FacetRefiner` and the
//! `substrated::Embedder` trait with no change to these interfaces.

mod router;

use anyhow::{bail, Result};
use clade_facets::Facet;
use router::{route, Dial, RouteInput};
use serde::Serialize;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("daemon") => clade_proto::run_service_stub("modeld"),
        Some("facets") => cmd_facets(&args[1..]),
        Some("route") => cmd_route(&args[1..]),
        Some(other) => {
            eprintln!("modeld: unknown command '{other}'");
            eprintln!(
                "usage: modeld [daemon]\n       \
                 modeld facets <path...> [--json]\n       \
                 modeld route [--local-confidence F] [--complexity F] [--sensitive] \
                 [--offline] [--dial airgapped|balanced|cloud-boosted] [--json]"
            );
            std::process::exit(2);
        }
    }
}

#[derive(Serialize)]
struct FileFacets {
    path: String,
    facets: Vec<Facet>,
}

fn cmd_facets(args: &[String]) -> Result<()> {
    let json = args.iter().any(|a| a == "--json");
    let roots: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    if roots.is_empty() {
        bail!("usage: modeld facets <path...> [--json]");
    }
    let mut files = Vec::new();
    for r in roots {
        collect_files(Path::new(r), &mut files);
    }
    let mut out = Vec::new();
    for path in files {
        let facets = clade_facets::resolve_path(&path).unwrap_or_default();
        out.push(FileFacets {
            path: path.to_string_lossy().into_owned(),
            facets,
        });
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        for ff in &out {
            let summary = ff
                .facets
                .iter()
                .map(|f| match &f.detail {
                    Some(d) => format!("{}:{d} {:.2}", f.kind, f.confidence),
                    None => format!("{} {:.2}", f.kind, f.confidence),
                })
                .collect::<Vec<_>>()
                .join(", ");
            println!(
                "{}\n    {}",
                ff.path,
                if summary.is_empty() { "—" } else { &summary }
            );
        }
    }
    Ok(())
}

/// Recurse directories into a flat file list (skips hidden entries).
fn collect_files(path: &Path, out: &mut Vec<PathBuf>) {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if name.starts_with('.') && !name.is_empty() {
        return;
    }
    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            let mut children: Vec<PathBuf> =
                entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
            children.sort();
            for c in children {
                collect_files(&c, out);
            }
        }
    } else if path.is_file() {
        out.push(path.to_path_buf());
    }
}

fn cmd_route(args: &[String]) -> Result<()> {
    let json = args.iter().any(|a| a == "--json");
    let input = RouteInput {
        local_confidence: flag_val(args, "--local-confidence")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0),
        complexity: flag_val(args, "--complexity")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0),
        sensitive: args.iter().any(|a| a == "--sensitive"),
        online: !args.iter().any(|a| a == "--offline"),
        dial: match flag_val(args, "--dial").as_deref() {
            Some("airgapped") => Dial::Airgapped,
            Some("cloud-boosted") => Dial::CloudBoosted,
            Some("balanced") | None => Dial::Balanced,
            Some(other) => bail!("unknown dial '{other}' (airgapped|balanced|cloud-boosted)"),
        },
    };
    let decision = route(input);
    if json {
        println!("{}", serde_json::to_string_pretty(&decision)?);
    } else {
        let engine = match decision.engine {
            router::Engine::Local => "LOCAL",
            router::Engine::Escalate => "ESCALATE",
        };
        println!("{engine}  (routing confidence {:.2})", decision.confidence);
        println!("  why: {}", decision.reason);
        if decision.redact {
            println!("  egress: redact sensitive spans before the request leaves");
        }
    }
    Ok(())
}

fn flag_val(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}
