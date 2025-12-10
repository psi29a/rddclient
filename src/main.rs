mod args;
mod clients;
mod config;
mod ip;
mod state;

use clap::CommandFactory;
use std::error::Error;

/// User-Agent header value for HTTP requests
pub const USER_AGENT: &str = concat!("rddclient/", env!("CARGO_PKG_VERSION"));

fn init_logger(verbose: bool, test: bool, debug: bool, quiet: bool) {
    let log_level = if quiet {
        log::LevelFilter::Error
    } else if debug {
        log::LevelFilter::Debug
    } else if verbose || test {
        log::LevelFilter::Info
    } else {
        log::LevelFilter::Warn
    };

    env_logger::builder()
        .filter(None, log_level)
        .init();
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::Args::new();
    let test = args.test;

    init_logger(args.verbose, test, args.debug, args.quiet);

    // Load and merge configuration
    let config = config::Config::load(&args)?;
    config.validate()?;

    // Display help if no host is configured
    if config.host.is_none() {
        println!("Missing required argument: host (use --host)");
        args::Args::command().print_help()?;
        return Ok(());
    }

    let protocol = config.protocol.as_ref()
        .ok_or("Protocol not specified (use --protocol)")?;

    log::info!("Starting {} DNS updater...", protocol);

    // Initialize state management
    let cache_path = args.cache.as_ref().map(std::path::PathBuf::from);
    let mut state_manager = state::StateManager::new(cache_path)?;

    // Get IP address
    let ip = ip::get_ip(config.ip.as_deref())?;
    log::info!("IP address: {}", ip);

    // Create the appropriate DNS client
    let client = clients::create_client(protocol, &config)?;
    client.validate_config()?;
    
    log::info!("Using provider: {}", client.provider_name());

    // Update each DNS record
    for hostname in config.dns_records() {
        // Check if IP has changed (unless --force is used)
        let host_state = state_manager.get(&hostname);
        if !args.force {
            if let Some(state) = host_state {
                if !state.ip_changed(ip) {
                    log::info!("{}: IP hasn't changed ({}), skipping update", hostname, ip);
                    continue;
                }
            }
        }
        
        if test {
            log::info!("TEST MODE: Would update {} to {}", hostname, ip);
            continue;
        }

        match client.update_record(&hostname, ip) {
            Ok(_) => {
                log::info!("Successfully updated {}", hostname);
                // Update state with success
                let state = state_manager.get_mut(&hostname);
                state.update_success(ip, "good".to_string());
            }
            Err(e) => {
                log::error!("Failed to update {}: {}", hostname, e);
                // Update state with failure
                let state = state_manager.get_mut(&hostname);
                state.update_failure(e.to_string());
            }
        }
    }

    // Save state to cache file
    state_manager.save()?;

    Ok(())
}
