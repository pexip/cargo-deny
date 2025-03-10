#![allow(clippy::exit)]

use anyhow::{bail, Context, Error};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

mod check;
mod common;
mod fetch;
mod init;
mod list;
mod stats;

#[derive(Subcommand, Debug)]
enum Command {
    /// Checks a project's crate graph
    #[clap(name = "check")]
    Check(check::Args),
    /// Fetches remote data
    #[clap(name = "fetch")]
    Fetch(fetch::Args),
    /// Creates a cargo-deny config from a template
    #[clap(name = "init")]
    Init(init::Args),
    /// Outputs a listing of all licenses and the crates that use them
    #[clap(name = "list")]
    List(list::Args),
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Format {
    Human,
    Json,
}

#[derive(ValueEnum, Copy, Clone, Debug)]
pub enum Color {
    Auto,
    Always,
    Never,
}

fn parse_level(s: &str) -> Result<log::LevelFilter, Error> {
    s.parse::<log::LevelFilter>()
        .with_context(|| format!("failed to parse level '{}'", s))
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub(crate) struct GraphContext {
    /// The path of a Cargo.toml to use as the context for the operation.
    ///
    /// By default, the Cargo.toml in the current working directory is used.
    #[clap(long, action)]
    pub(crate) manifest_path: Option<PathBuf>,
    /// If passed, all workspace packages are used as roots for the crate graph.
    ///
    /// Automatically assumed if the manifest path points to a virtual manifest.
    ///
    /// Normally, if you specify a manifest path that is a member of a workspace, that crate will be the sole root of the crate graph, meaning only other workspace members that are dependencies of that workspace crate will be included in the graph. This overrides that behavior to include all workspace members.
    #[clap(long, action)]
    pub(crate) workspace: bool,
    /// One or more crates to exclude from the crate graph that is used.
    ///
    /// NOTE: Unlike cargo, this does not have to be used with the `--workspace` flag.
    #[clap(long, action)]
    pub(crate) exclude: Vec<String>,
    /// One or more platforms to filter crates by
    ///
    /// If a dependency is target specific, it will be ignored if it does not match 1 or more of the specified targets. This option overrides the top-level `targets = []` configuration value.
    #[clap(short, long, action)]
    pub(crate) target: Vec<String>,
    /// Activate all available features
    #[clap(long, action)]
    pub(crate) all_features: bool,
    /// Do not activate the `default` feature
    #[clap(long, action)]
    pub(crate) no_default_features: bool,
    /// Space or comma separated list of features to activate
    #[clap(long, use_value_delimiter = true, action)]
    pub(crate) features: Vec<String>,
    /// Require Cargo.lock and cache are up to date
    #[clap(long, action)]
    pub(crate) frozen: bool,
    /// Require Cargo.lock is up to date
    #[clap(long, action)]
    pub(crate) locked: bool,
    /// Run without accessing the network. If used with the `check` subcommand, this also disables advisory database fetching.
    #[clap(long, action)]
    pub(crate) offline: bool,
}

/// Lints your project's crate graph
#[derive(Parser)]
#[clap(author, version, about, long_about = None, rename_all = "kebab-case", max_term_width = 80)]
struct Opts {
    /// The log level for messages
    #[clap(
        short = 'L',
        long = "log-level",
        default_value = "warn",
        value_parser = parse_level,
        long_help = "The log level for messages

Only log messages at or above the level will be emitted.

Possible values:
* off
* error
* warn
* info
* debug
* trace
")]
    log_level: log::LevelFilter,
    /// Specify the format of cargo-deny's output
    #[clap(short, long, default_value = "human", value_enum, action)]
    format: Format,
    #[clap(
        short,
        long,
        default_value = "auto",
        value_enum,
        env = "CARGO_TERM_COLOR",
        action
    )]
    color: Color,
    #[clap(flatten)]
    ctx: GraphContext,
    #[clap(subcommand)]
    cmd: Command,
}

fn setup_logger(
    level: log::LevelFilter,
    format: Format,
    color: bool,
) -> Result<(), fern::InitError> {
    use ansi_term::Color::{Blue, Green, Purple, Red, Yellow};
    use log::Level::{Debug, Error, Info, Trace, Warn};

    let now = time::OffsetDateTime::now_utc();

    match format {
        Format::Human => {
            const HUMAN: &[time::format_description::FormatItem<'static>] =
                time::macros::format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

            if color {
                fern::Dispatch::new()
                    .level(level)
                    .format(move |out, message, record| {
                        out.finish(format_args!(
                            "{date} [{level}] {message}\x1B[0m",
                            date = now.format(&HUMAN).unwrap(),
                            level = match record.level() {
                                Error => Red.paint("ERROR"),
                                Warn => Yellow.paint("WARN"),
                                Info => Green.paint("INFO"),
                                Debug => Blue.paint("DEBUG"),
                                Trace => Purple.paint("TRACE"),
                            },
                            message = message,
                        ));
                    })
                    .chain(std::io::stderr())
                    .apply()?;
            } else {
                fern::Dispatch::new()
                    .level(level)
                    .format(move |out, message, record| {
                        out.finish(format_args!(
                            "{date} [{level}] {message}",
                            date = now.format(&HUMAN).unwrap(),
                            level = match record.level() {
                                Error => "ERROR",
                                Warn => "WARN",
                                Info => "INFO",
                                Debug => "DEBUG",
                                Trace => "TRACE",
                            },
                            message = message,
                        ));
                    })
                    .chain(std::io::stderr())
                    .apply()?;
            }
        }
        Format::Json => {
            fern::Dispatch::new()
                .level(level)
                .format(move |out, message, record| {
                    out.finish(format_args!(
                        "{}",
                        serde_json::json! {{
                            "type": "log",
                            "fields": {
                                "timestamp": now.format(&time::format_description::well_known::Rfc3339).unwrap(),
                                "level": match record.level() {
                                    Error => "ERROR",
                                    Warn => "WARN",
                                    Info => "INFO",
                                    Debug => "DEBUG",
                                    Trace => "TRACE",
                                },
                                "message": message,
                            }
                        }}
                    ));
                })
                .chain(std::io::stderr())
                .apply()?;
        }
    }

    Ok(())
}

fn real_main() -> Result<(), Error> {
    let args =
        Opts::parse_from({
            std::env::args().enumerate().filter_map(|(i, a)| {
                if i == 1 && a == "deny" {
                    None
                } else {
                    Some(a)
                }
            })
        });

    let log_level = args.log_level;

    let color = match args.color {
        Color::Auto => atty::is(atty::Stream::Stderr),
        Color::Always => true,
        Color::Never => false,
    };

    setup_logger(log_level, args.format, color)?;

    let manifest_path = if let Some(mpath) = args.ctx.manifest_path {
        mpath
    } else {
        // For now, use the context path provided by the user, but
        // we've deprected it and it will go away at some point
        let cwd =
            std::env::current_dir().context("unable to determine current working directory")?;

        if !cwd.exists() {
            bail!("current working directory {} was not found", cwd.display());
        }

        if !cwd.is_dir() {
            bail!(
                "current working directory {} is not a directory",
                cwd.display()
            );
        }

        let man_path = cwd.join("Cargo.toml");

        if !man_path.exists() {
            bail!(
                "the directory {} doesn't contain a Cargo.toml file",
                cwd.display()
            );
        }

        man_path
    };

    if manifest_path.file_name() != Some(std::ffi::OsStr::new("Cargo.toml"))
        || !manifest_path.is_file()
    {
        bail!("--manifest-path must point to a Cargo.toml file");
    }

    if !manifest_path.exists() {
        bail!("unable to find cargo manifest {}", manifest_path.display());
    }

    let krate_ctx = common::KrateContext {
        manifest_path,
        workspace: args.ctx.workspace,
        exclude: args.ctx.exclude,
        targets: args.ctx.target,
        no_default_features: args.ctx.no_default_features,
        all_features: args.ctx.all_features,
        features: args.ctx.features,
        frozen: args.ctx.frozen,
        locked: args.ctx.locked,
        offline: args.ctx.offline,
    };

    let log_ctx = crate::common::LogContext {
        color: args.color,
        format: args.format,
        log_level: args.log_level,
    };

    match args.cmd {
        Command::Check(mut cargs) => {
            let show_stats = cargs.show_stats;

            if args.ctx.offline {
                log::info!("network access disabled via --offline flag, disabling advisory database fetching");
                cargs.disable_fetch = true;
            }

            let stats = check::cmd(log_ctx, cargs, krate_ctx)?;

            let errors = stats.total_errors();

            stats::print_stats(stats, show_stats, log_level, args.format, args.color);

            if errors > 0 {
                std::process::exit(1);
            } else {
                Ok(())
            }
        }
        Command::Fetch(fargs) => fetch::cmd(log_ctx, fargs, krate_ctx),
        Command::Init(iargs) => init::cmd(iargs, krate_ctx),
        Command::List(largs) => list::cmd(log_ctx, largs, krate_ctx),
    }
}

fn main() {
    match real_main() {
        Ok(_) => {}
        Err(e) => {
            log::error!("{:#}", e);
            std::process::exit(1);
        }
    }
}
