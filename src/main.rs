mod args;
mod clients;
mod config;
mod ip;

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

    // Get IP address
    let ip = ip::get_ip(config.ip.as_deref())?;
    log::info!("IP address: {}", ip);

    // Create the appropriate DNS client
    let client = clients::create_client(protocol, &config)?;
    client.validate_config()?;
    
    log::info!("Using provider: {}", client.provider_name());

    // Update each DNS record
    for hostname in config.dns_records() {
        if test {
            log::info!("TEST MODE: Would update {} to {}", hostname, ip);
            continue;
        }

        match client.update_record(&hostname, ip) {
            Ok(_) => log::info!("Successfully updated {}", hostname),
            Err(e) => log::error!("Failed to update {}: {}", hostname, e),
        }
    }

    Ok(())
}
