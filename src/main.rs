#[macro_use]
mod util;
mod autocomplete;
mod cli;
mod error;
mod icon;
mod parser;
mod prompt;
mod runtime;

use clap::Parser;
use cli::Command;
use prompt::YesOrNo;
use runtime::{CheckerContext, Runtime};
use thisctx::WithContext;
use tracing::{error, Level};

static CACHED: &str = include_str!("./cached.txt");

fn main_impl() -> error::Result<()> {
    let args = cli::Cli::parse();

    let lv = match args.verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(lv)
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber).context(error::Any)?;

    let mut rt = Runtime::builder();
    if args.input.is_empty() {
        rt.load_cache(CACHED);
    } else {
        for path in args.input.iter() {
            rt.load_input(path)?;
        }
    }
    let rt = rt.build();
    match args.cmd {
        Command::Cache { output } => rt.save_cache(&output)?,
        Command::Check { source, format } => {
            let mut context = CheckerContext {
                format,
                ..Default::default()
            };
            for path in source.iter() {
                log_or_break!(rt.check(&mut context, path, false));
            }
        }
        Command::Fix {
            source,
            yes,
            replace,
        } => {
            let mut context = CheckerContext {
                replace,
                yes,
                ..Default::default()
            };
            for path in source.iter() {
                log_or_break!((|| {
                    if let Some(patched) = rt.check(&mut context, path, true)? {
                        if !context.yes {
                            match prompt::prompt_yes_or_no(
                                "Are your sure to write the patched content?",
                                None,
                            )? {
                                YesOrNo::No => return Ok(()),
                                YesOrNo::AllYes => context.yes = true,
                                _ => {}
                            }
                        }
                        std::fs::write(path, patched)
                            .context_with(|| error::Io(path.to_owned()))?;
                    }
                    Ok(())
                })());
            }
        }
        Command::Search {} => {
            rt.prompt_input_icon(None).ok();
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = main_impl() {
        log_error!(e);
    }
}
