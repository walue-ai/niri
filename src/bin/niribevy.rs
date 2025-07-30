use std::env;
use std::path::PathBuf;

use calloop::EventLoop;
use clap::Parser;
use smithay::reexports::wayland_server::Display;
use tracing::info;
use tracing_subscriber::EnvFilter;

use niri::cli::{Cli, Sub};
use niri::ipc::client::handle_msg;
use niri::niri::State;
use niri::utils::version;
use niri_config::Config;

const DEFAULT_LOG_FILTER: &str = "niri=debug,smithay::backend::renderer::gles=error";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set backtrace defaults if not set.
    if env::var_os("RUST_BACKTRACE").is_none() {
        env::set_var("RUST_BACKTRACE", "1");
    }
    if env::var_os("RUST_LIB_BACKTRACE").is_none() {
        env::set_var("RUST_LIB_BACKTRACE", "0");
    }

    let directives = env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_LOG_FILTER.to_owned());
    let env_filter = EnvFilter::builder().parse_lossy(directives);
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(env_filter)
        .init();

    info!("Starting niribevy - Niri with Bevy integration");
    info!("Version: {}", version());

    let cli = Cli::parse();

    if let Some(subcommand) = cli.subcommand {
        match subcommand {
            Sub::Validate { config } => {
                let (path, _, _) = config_path(config);
                Config::load(&path)?;
                info!("config is valid");
                return Ok(());
            }
            Sub::Msg { msg, json } => {
                handle_msg(msg, json)?;
                return Ok(());
            }
            Sub::Panic => {
                panic!("Intentional panic for testing");
            }
            Sub::Completions { shell: _ } => {
                info!("Completions not implemented for niribevy");
                return Ok(());
            }
        }
    }

    info!("Starting niribevy daemon with Bevy integration enabled");
    
    env::set_var("NIRI_BEVY_ENABLED", "1");
    
    let mut event_loop = EventLoop::<State>::try_new().unwrap();
    
    let (path, _, _) = config_path(cli.config);
    let config = Config::load(&path).unwrap_or_default();
    
    let display = Display::new().unwrap();
    let mut state = State::new(
        config,
        event_loop.handle(),
        event_loop.get_signal(),
        display,
        false,
        true,
        cli.session,
    )?;
    
    info!("niribevy compositor started with Bevy integration");
    
    event_loop
        .run(None, &mut state, |state| state.refresh_and_flush_clients())
        .unwrap();

    Ok(())
}

/// Resolves and returns the config path to load, the config path to watch, and whether to create
/// the default config at the path to load.
fn config_path(cli_path: Option<PathBuf>) -> (PathBuf, PathBuf, bool) {
    if let Some(explicit) = cli_path {
        return (explicit.clone(), explicit, false);
    }

    let default_path = PathBuf::from("config.kdl");
    (default_path.clone(), default_path, false)
}
