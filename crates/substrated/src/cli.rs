//! The demo CLI. Transport: try the daemon's control socket first (single
//! writer preserved, warm cache); if no daemon is up, open the DB directly so
//! `substrated query photo.jpg` works standalone.

use crate::embed::PhashHistEmbedder;
use crate::indexer::Indexer;
use crate::search::{self, QueryHit};
use crate::{config, ctl, db};
use anyhow::{bail, Result};

/// Dispatch a CLI subcommand. `args[0]` is the subcommand.
pub fn run(args: &[String]) -> Result<()> {
    let sub = args[0].as_str();
    let rest = &args[1..];
    let json = rest.iter().any(|a| a == "--json");
    match sub {
        "query" => cmd_query(rest, json),
        "list" => cmd_list(rest, json),
        "stats" => cmd_stats(json),
        "reindex" => cmd_reindex(json),
        other => bail!("unknown command: {other}"),
    }
}

fn cmd_query(rest: &[String], json: bool) -> Result<()> {
    let path = rest
        .iter()
        .find(|a| !a.starts_with("--"))
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!("usage: substrated query <path> [--top N] [--time-aware]")
        })?;
    let top = flag_val(rest, "--top")
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);
    let time_aware = rest.iter().any(|a| a == "--time-aware");
    // Resolve to an absolute path so it matches the stored (absolute) keys.
    let abs = std::fs::canonicalize(&path)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or(path);

    let hits = match ctl::request(
        &config::ctl_path(),
        &ctl::Request::Query {
            path: abs.clone(),
            top,
            time_aware,
        },
    )? {
        Some(ctl::Reply::Query(h)) => h,
        Some(ctl::Reply::Error(e)) => bail!(e),
        Some(_) => bail!("unexpected reply"),
        None => {
            // Standalone: open the DB directly (read-only).
            let conn = db::open_ro(&config::db_path())?;
            let cache = db::load_cache(&conn, db::EMBED_MODEL)?;
            search::query_neighbors(&conn, &cache, &abs, top, time_aware)?
        }
    };
    print_hits(&hits, json, true);
    Ok(())
}

fn cmd_list(rest: &[String], json: bool) -> Result<()> {
    let limit = flag_val(rest, "--limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(50);
    let hits = match ctl::request(&config::ctl_path(), &ctl::Request::List { limit })? {
        Some(ctl::Reply::List(h)) => h,
        Some(ctl::Reply::Error(e)) => bail!(e),
        Some(_) => bail!("unexpected reply"),
        None => {
            let conn = db::open_ro(&config::db_path())?;
            search::briefs_to_hits(db::list_items(&conn, limit)?)
        }
    };
    print_hits(&hits, json, false);
    Ok(())
}

fn cmd_stats(json: bool) -> Result<()> {
    let stats = match ctl::request(&config::ctl_path(), &ctl::Request::Stats)? {
        Some(ctl::Reply::Stats(s)) => s,
        Some(ctl::Reply::Error(e)) => bail!(e),
        Some(_) => bail!("unexpected reply"),
        None => db::stats(&db::open_ro(&config::db_path())?)?,
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        println!(
            "items {}  files {}  embeddings {}  failed {}  (schema {} · model {})",
            stats.items,
            stats.files,
            stats.embeddings,
            stats.failed,
            stats.schema_version,
            stats.embed_model
        );
    }
    Ok(())
}

fn cmd_reindex(json: bool) -> Result<()> {
    let stats = match ctl::request(&config::ctl_path(), &ctl::Request::Reindex)? {
        Some(ctl::Reply::Report(s)) => s,
        Some(ctl::Reply::Error(e)) => bail!(e),
        Some(_) => bail!("unexpected reply"),
        None => {
            // Standalone: we are the only process; scan directly.
            let mut ix = Indexer::open(&config::db_path(), Box::new(PhashHistEmbedder::new()))?;
            let report = ix.scan_root(&config::library_path())?;
            crate::log("substrated", &format!("reindex: {report:?}"));
            db::stats(ix.connection())?
        }
    };
    cmd_stats_from(stats, json);
    Ok(())
}

fn cmd_stats_from(stats: db::Stats, json: bool) {
    if json {
        println!("{}", serde_json::to_string(&stats).unwrap_or_default());
    } else {
        println!(
            "reindexed → items {}  embeddings {}",
            stats.items, stats.embeddings
        );
    }
}

fn print_hits(hits: &[QueryHit], json: bool, with_score: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(hits).unwrap_or_default());
        return;
    }
    if hits.is_empty() {
        println!("(no results)");
        return;
    }
    for h in hits {
        if with_score {
            println!("{:.4}  {}  {}", h.score, h.sub_id, h.path);
        } else {
            println!("{}  {}", h.sub_id, h.path);
        }
    }
}

fn flag_val(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}
