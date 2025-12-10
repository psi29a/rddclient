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

    // Parse rate limiting intervals (defaults match ddclient)
    let min_interval = args.min_interval.as_deref()
        .map(config::parse_interval)
        .transpose()?
        .or(Some(30)); // Default: 30 seconds
    
    let max_interval = args.max_interval.as_deref()
        .map(config::parse_interval)
        .transpose()?
        .or(Some(25 * 86400)); // Default: 25 days
    
    let min_error_interval = args.min_error_interval.as_deref()
        .map(config::parse_interval)
        .transpose()?
        .or(Some(300)); // Default: 5 minutes

    // Determine IP detection method
    let detection_method = if let Some(ip_str) = config.ip.as_deref() {
        ip::IpDetectionMethod::Manual(ip_str.to_string())
    } else if let Some(use_method) = args.use_method.as_deref() {
        match use_method {
            "ip" => {
                return Err("--use=ip requires --ip parameter".into());
            }
            "web" => ip::IpDetectionMethod::Web(args.web.clone()),
            "if" => {
                let iface = args.if_name.as_deref()
                    .ok_or("--use=if requires --if parameter")?;
                ip::IpDetectionMethod::Interface(iface.to_string())
            }
            "cmd" => {
                let cmd = args.cmd.as_deref()
                    .ok_or("--use=cmd requires --cmd parameter")?;
                ip::IpDetectionMethod::Command(cmd.to_string())
            }
            _ => {
                return Err(format!("Unknown IP detection method: {}", use_method).into());
            }
        }
    } else {
        ip::IpDetectionMethod::Web(None)
    };

    // Get IP address using the chosen method
    let ip = ip::get_ip_with_method(&detection_method)?;
    log::info!("IP address: {} (detected via {:?})", ip, detection_method);

    // Create the appropriate DNS client
    let client = clients::create_client(protocol, &config)?;
    client.validate_config()?;
    
    log::info!("Using provider: {}", client.provider_name());

    // Update each DNS record
    for hostname in config.dns_records() {
        // Check if IP has changed
        let host_state = state_manager.get(&hostname);
        let ip_changed = host_state.is_none_or(|state| state.ip_changed(ip));
        
        // Check rate limits
        let (should_update, skip_reason) = state_manager.should_update(
            &hostname,
            ip_changed,
            args.force,
            min_interval,
            max_interval,
            min_error_interval,
        );
        
        if !should_update {
            if let Some(reason) = skip_reason {
                log::info!("{}: {}", hostname, reason);
            }
            continue;
        }
        
        // Log if update was forced
        if let Some(reason) = skip_reason {
            log::info!("{}: {}", hostname, reason);
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
