use clap::{CommandFactory, Parser};
use std::io;
use std::process;
use tracing_subscriber::EnvFilter;

use jev_ops::cli::{Cli, Commands, PacksCommands, VERSION_STRING};
use jev_ops::engine;
use jev_ops::error::Result;
use jev_ops::output;
use jev_ops::packs;

fn setup_logging(verbosity: u8) {
    let filter = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter)),
        )
        .with_writer(io::stderr)
        .try_init();
}

fn main() {
    // clap prints --help/--version to stdout with exit 0, and usage errors to stderr with exit 2.
    let cli = Cli::parse();

    setup_logging(cli.verbose);

    if let Err(err) = run(cli) {
        eprintln!("error: {}", err);
        process::exit(err.exit_code());
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Version => {
            println!("jev-ops {}", VERSION_STRING);
            Ok(())
        }
        Commands::Completion { shell } => {
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "jev-ops", &mut io::stdout());
            Ok(())
        }
        Commands::Analyze {
            pack,
            input,
            json,
            pack_dir,
            min_confidence,
            provider,
            api_key,
        } => {
            let result = engine::pipeline::run_pipeline(
                &pack,
                input.as_deref(),
                pack_dir.as_deref(),
                min_confidence,
                provider.as_deref(),
                api_key.as_deref(),
            )?;

            if json {
                let json_output = output::json::render_json(&result)?;
                println!("{}", json_output);
            } else {
                let human_output = output::human::render_human(&result);
                print!("{}", human_output);
            }
            Ok(())
        }
        Commands::Packs { command } => match command {
            PacksCommands::List { pack_dir, json } => {
                let available_packs = packs::discovery::list_available_packs(pack_dir.as_deref());

                if json {
                    let items: Vec<serde_json::Value> = available_packs
                        .iter()
                        .map(|p| {
                            serde_json::json!({
                                "name": p.name,
                                "version": p.manifest.metadata.version,
                                "description": p.manifest.metadata.description,
                                "author": p.manifest.metadata.author,
                                "path": p.path.display().to_string(),
                            })
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&items)?);
                } else if available_packs.is_empty() {
                    println!("No diagnostic packs found.");
                } else {
                    println!("AVAILABLE DIAGNOSTIC PACKS:");
                    println!(
                        "{:<18} {:<10} {:<40} PATH",
                        "NAME", "VERSION", "DESCRIPTION"
                    );
                    println!("{}", "─".repeat(80));
                    for p in available_packs {
                        let desc = truncate_chars(&p.manifest.metadata.description, 38);
                        println!(
                            "{:<18} {:<10} {:<40} {}",
                            p.name,
                            p.manifest.metadata.version,
                            desc,
                            p.path.display()
                        );
                    }
                }
                Ok(())
            }
            PacksCommands::Show {
                pack,
                pack_dir,
                json,
            } => {
                let (path, manifest) = packs::discovery::find_pack(&pack, pack_dir.as_deref())?;

                if json {
                    println!("{}", serde_json::to_string_pretty(&manifest)?);
                } else {
                    println!("Diagnostic Pack: {}", manifest.metadata.name);
                    println!("Version:         {}", manifest.metadata.version);
                    println!("Author:          {}", manifest.metadata.author);
                    println!("Path:            {}", path.display());
                    println!("Description:     {}", manifest.metadata.description);
                    println!("\nSpec Input:");
                    println!("  Type:          {}", manifest.spec.input.input_type);
                    if let Some(max_b) = manifest.spec.input.max_bytes {
                        println!("  Max Bytes:     {}", max_b);
                    }
                    println!("\nDecisions ({} defined):", manifest.spec.decisions.len());
                    for (name, spec) in &manifest.spec.decisions {
                        match spec {
                            packs::manifest::DecisionSpec::Choice { values } => {
                                println!("  - {}: choice (options: {})", name, values.join(", "));
                            }
                            packs::manifest::DecisionSpec::Score { min, max, levels } => {
                                println!("  - {}: score (range: {} to {})", name, min, max);
                                for (value, level) in (*min..=*max).zip(levels) {
                                    println!("      {}: {}", value, level);
                                }
                            }
                            packs::manifest::DecisionSpec::Boolean => {
                                println!("  - {}: boolean (yes/no)", name);
                            }
                        }
                    }
                    println!("\nInstructions:\n{}", manifest.spec.instructions.trim());
                }
                Ok(())
            }
            PacksCommands::Validate { path } => {
                let manifest = packs::loader::load_manifest(&path)?;
                println!(
                    "✓ Pack '{}' ({}) at '{}' is valid.",
                    manifest.metadata.name,
                    manifest.metadata.version,
                    path.display()
                );
                Ok(())
            }
            PacksCommands::New { name, dir } => {
                let path = packs::scaffold::scaffold_pack(&name, dir.as_deref())?;
                println!(
                    "✓ Scaffolded diagnostic pack template at '{}'",
                    path.display()
                );
                Ok(())
            }
        },
    }
}

/// Shortens `text` to at most `max` characters, ending with "..." when cut.
/// Counts chars, not bytes, so multi-byte UTF-8 descriptions never split mid-character.
fn truncate_chars(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let kept: String = text.chars().take(max.saturating_sub(3)).collect();
    format!("{}...", kept)
}
