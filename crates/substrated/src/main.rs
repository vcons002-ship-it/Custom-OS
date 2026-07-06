//! substrated entry point. No args (or "daemon") → the supervised daemon that
//! weaved launches. A subcommand → the demo CLI.

use substrated::{cli, daemon};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("daemon") => daemon::run(),
        Some("query") | Some("list") | Some("stats") | Some("reindex") => cli::run(&args),
        Some(other) => {
            eprintln!("substrated: unknown command '{other}'");
            eprintln!("usage: substrated [daemon] | query <path> [--top N] [--time-aware] | list | stats | reindex");
            std::process::exit(2);
        }
    }
}
