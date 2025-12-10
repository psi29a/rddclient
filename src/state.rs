/// State management for tracking IP addresses and update history
///
/// This module handles persistent state storage to enable:
/// - IP change detection (only update when IP changes)
/// - Update history tracking
/// - Error tracking and retry logic
///
/// The state file (cache) is stored in:
/// - Linux/macOS: /var/cache/rddclient/<host>.cache or ~/.cache/rddclient/<host>.cache
/// - Windows: %LOCALAPPDATA%\rddclient\cache\<host>.cache
///
/// Format is ddclient-compatible: simple key=value pairs per hostname
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// State for a single host
#[derive(Debug, Clone)]
pub struct HostState {
    /// Last known IP address
    pub ip: Option<IpAddr>,
    
    /// Last successful update timestamp (Unix epoch seconds)
    pub mtime: Option<u64>,
    
    /// Last update status (e.g., "good", "nochg", error message)
    pub status: Option<String>,
    
    /// Number of consecutive update failures
    pub atime: Option<u64>,  // ddclient calls this "atime" (access time for error tracking)
    
    /// Warning counter
    pub wtime: Option<u64>,  // ddclient uses this for warning tracking
}

impl HostState {
    pub fn new() -> Self {
        Self {
            ip: None,
            mtime: None,
            status: None,
            atime: None,
            wtime: None,
        }
    }
    
    /// Check if IP has changed
    pub fn ip_changed(&self, new_ip: IpAddr) -> bool {
        match self.ip {
            Some(cached_ip) => cached_ip != new_ip,
            None => true,  // No cached IP means we should update
        }
    }
    
    /// Update state after successful DNS update
    pub fn update_success(&mut self, ip: IpAddr, status: String) {
        self.ip = Some(ip);
        self.mtime = Some(current_timestamp());
        self.status = Some(status);
        self.atime = None;  // Reset error counter on success
    }
    
    /// Update state after failed DNS update
    pub fn update_failure(&mut self, error: String) {
        self.status = Some(format!("FAILED: {}", error));
        self.atime = Some(current_timestamp());
    }
}

impl Default for HostState {
    fn default() -> Self {
        Self::new()
    }
}

/// State manager - handles reading/writing cache file
pub struct StateManager {
    cache_file: PathBuf,
    states: HashMap<String, HostState>,
}

impl StateManager {
    /// Create new state manager with cache file path
    pub fn new(cache_file: Option<PathBuf>) -> Result<Self, Box<dyn Error>> {
        let cache_file = match cache_file {
            Some(path) => path,
            None => Self::default_cache_path()?,
        };
        
        let mut manager = Self {
            cache_file,
            states: HashMap::new(),
        };
        
        // Try to load existing state
        if manager.cache_file.exists() {
            manager.load()?;
        }
        
        Ok(manager)
    }
    
    /// Get default cache file path based on platform
    fn default_cache_path() -> Result<PathBuf, Box<dyn Error>> {
        #[cfg(target_os = "linux")]
        {
            // Try /var/cache/rddclient first, fall back to user cache
            let system_cache = PathBuf::from("/var/cache/rddclient/rddclient.cache");
            if let Some(parent) = system_cache.parent() {
                if parent.exists() || fs::create_dir_all(parent).is_ok() {
                    return Ok(system_cache);
                }
            }
            
            // Fall back to user cache directory
            if let Some(cache_dir) = dirs::cache_dir() {
                let user_cache = cache_dir.join("rddclient").join("rddclient.cache");
                return Ok(user_cache);
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            // Try /var/cache/rddclient first
            let system_cache = PathBuf::from("/var/cache/rddclient/rddclient.cache");
            if let Some(parent) = system_cache.parent() {
                if parent.exists() || fs::create_dir_all(parent).is_ok() {
                    return Ok(system_cache);
                }
            }
            
            // Fall back to ~/Library/Caches/rddclient
            if let Some(home) = dirs::home_dir() {
                let user_cache = home
                    .join("Library")
                    .join("Caches")
                    .join("rddclient")
                    .join("rddclient.cache");
                return Ok(user_cache);
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            if let Some(local_appdata) = dirs::cache_dir() {
                let cache = local_appdata.join("rddclient").join("cache").join("rddclient.cache");
                return Ok(cache);
            }
        }
        
        Err("Failed to determine cache file location".into())
    }
    
    /// Get state for a specific host
    pub fn get(&self, hostname: &str) -> Option<&HostState> {
        self.states.get(hostname)
    }
    
    /// Get mutable state for a specific host (creates if doesn't exist)
    pub fn get_mut(&mut self, hostname: &str) -> &mut HostState {
        self.states.entry(hostname.to_string()).or_default()
    }
    
    /// Load state from cache file (ddclient format)
    pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
        let content = fs::read_to_string(&self.cache_file)?;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with("##") {
                continue;
            }
            
            // Parse: key1=value1,key2=value2,... hostname
            // Find last space to separate options from hostname
            if let Some(last_space) = line.rfind(' ') {
                let (opts_str, hostname) = line.split_at(last_space);
                let hostname = hostname.trim();
                
                if hostname.is_empty() {
                    continue;
                }
                
                let mut state = HostState::new();
                
                // Parse key=value pairs
                for pair in opts_str.split(',') {
                    let pair = pair.trim();
                    if let Some((key, value)) = pair.split_once('=') {
                        let key = key.trim();
                        let value = value.trim();
                        
                        match key {
                            "ip" => {
                                if let Ok(ip) = value.parse::<IpAddr>() {
                                    state.ip = Some(ip);
                                }
                            }
                            "mtime" => {
                                if let Ok(timestamp) = value.parse::<u64>() {
                                    state.mtime = Some(timestamp);
                                }
                            }
                            "status" => {
                                state.status = Some(value.to_string());
                            }
                            "atime" => {
                                if let Ok(timestamp) = value.parse::<u64>() {
                                    state.atime = Some(timestamp);
                                }
                            }
                            "wtime" => {
                                if let Ok(timestamp) = value.parse::<u64>() {
                                    state.wtime = Some(timestamp);
                                }
                            }
                            _ => {}  // Ignore unknown keys
                        }
                    }
                }
                
                self.states.insert(hostname.to_string(), state);
            }
        }
        
        Ok(())
    }
    
    /// Save state to cache file (ddclient format)
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.cache_file.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = fs::File::create(&self.cache_file)?;
        
        // Write header
        writeln!(file, "## rddclient cache file")?;
        writeln!(file, "## last updated at {} ({})", 
                 format_timestamp(current_timestamp()), 
                 current_timestamp())?;
        
        // Write each host state
        for (hostname, state) in &self.states {
            let mut parts = Vec::new();
            
            if let Some(ip) = state.ip {
                parts.push(format!("ip={}", ip));
            }
            if let Some(mtime) = state.mtime {
                parts.push(format!("mtime={}", mtime));
            }
            if let Some(status) = &state.status {
                parts.push(format!("status={}", status));
            }
            if let Some(atime) = state.atime {
                parts.push(format!("atime={}", atime));
            }
            if let Some(wtime) = state.wtime {
                parts.push(format!("wtime={}", wtime));
            }
            
            if !parts.is_empty() {
                writeln!(file, "{} {}", parts.join(","), hostname)?;
            }
        }
        
        Ok(())
    }

    /// Check if an update should be allowed based on rate limits
    /// Returns (should_update, reason_if_skipped)
    pub fn should_update(
        &self,
        hostname: &str,
        ip_changed: bool,
        force: bool,
        min_interval: Option<u64>,      // seconds
        max_interval: Option<u64>,      // seconds
        min_error_interval: Option<u64>, // seconds
    ) -> (bool, Option<String>) {
        // Always allow if force flag is set
        if force {
            return (true, None);
        }

        let state = self.states.get(hostname);
        if state.is_none() {
            // No previous state, allow update
            return (true, None);
        }

        let state = state.unwrap();
        let now = current_timestamp();

        // Check max-interval: Force update if too much time has passed since last successful update
        if let (Some(mtime), Some(max_int)) = (state.mtime, max_interval) {
            if now >= mtime + max_int {
                let days = max_int / 86400;
                return (true, Some(format!(
                    "update forced because it has been {} days since the previous update",
                    days
                )));
            }
        }

        // If IP hasn't changed and we're not being forced, no need to update
        if !ip_changed {
            return (false, Some("IP address hasn't changed".to_string()));
        }

        // IP has changed - check min-interval for successful updates
        if let Some(status) = &state.status {
            if status.starts_with("good") || status.starts_with("nochg") {
                // Last update was successful, check min-interval
                if let (Some(mtime), Some(min_int)) = (state.mtime, min_interval) {
                    if now < mtime + min_int {
                        let remaining = (mtime + min_int) - now;
                        return (false, Some(format!(
                            "skipped update due to min-interval ({}s remaining)",
                            remaining
                        )));
                    }
                }
            } else {
                // Last update failed, check min-error-interval
                if let (Some(atime), Some(min_err_int)) = (state.atime, min_error_interval) {
                    if now < atime + min_err_int {
                        let remaining = (atime + min_err_int) - now;
                        return (false, Some(format!(
                            "skipped update due to min-error-interval ({}s remaining after previous failure)",
                            remaining
                        )));
                    }
                }
            }
        }

        // All checks passed, allow update
        (true, None)
    }
}

/// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before Unix epoch")
        .as_secs()
}

/// Format timestamp as human-readable string
fn format_timestamp(timestamp: u64) -> String {
    // Simple UTC format - could use chrono for better formatting
    let datetime = UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
    format!("{:?}", datetime)  // Basic debug format for now
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_host_state_ip_changed() {
        let mut state = HostState::new();
        let ip1 = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
        let ip2 = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 5));
        
        // No cached IP = changed
        assert!(state.ip_changed(ip1));
        
        // Update with IP
        state.ip = Some(ip1);
        
        // Same IP = not changed
        assert!(!state.ip_changed(ip1));
        
        // Different IP = changed
        assert!(state.ip_changed(ip2));
    }
    
    #[test]
    fn test_host_state_update_success() {
        let mut state = HostState::new();
        let ip = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
        
        state.update_success(ip, "good".to_string());
        
        assert_eq!(state.ip, Some(ip));
        assert_eq!(state.status, Some("good".to_string()));
        assert!(state.mtime.is_some());
        assert!(state.atime.is_none());  // Error counter reset
    }
    
    #[test]
    fn test_state_manager_save_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let cache_path = temp_file.path().to_path_buf();
        
        // Create and populate state
        {
            let mut manager = StateManager::new(Some(cache_path.clone())).unwrap();
            let state = manager.get_mut("example.com");
            state.update_success(
                IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)),
                "good".to_string()
            );
            manager.save().unwrap();
        }
        
        // Load state in new manager
        {
            let manager = StateManager::new(Some(cache_path)).unwrap();
            let state = manager.get("example.com").unwrap();
            assert_eq!(state.ip, Some(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))));
            assert_eq!(state.status, Some("good".to_string()));
        }
    }
    
    #[test]
    fn test_parse_cache_format() {
        let temp_file = NamedTempFile::new().unwrap();
        let cache_path = temp_file.path();
        
        // Write ddclient-compatible format
        let content = "\
## rddclient cache file
## last updated at 2024-01-01 (1704067200)
ip=1.2.3.4,mtime=1704067200,status=good example.com
ip=5.6.7.8,mtime=1704067201,status=nochg www.example.com
";
        fs::write(cache_path, content).unwrap();
        
        // Load and verify
        let manager = StateManager::new(Some(cache_path.to_path_buf())).unwrap();
        
        let state1 = manager.get("example.com").unwrap();
        assert_eq!(state1.ip, Some(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4))));
        assert_eq!(state1.status, Some("good".to_string()));
        
        let state2 = manager.get("www.example.com").unwrap();
        assert_eq!(state2.ip, Some(IpAddr::V4(Ipv4Addr::new(5, 6, 7, 8))));
        assert_eq!(state2.status, Some("nochg".to_string()));
    }

    #[test]
    fn test_should_update_no_state() {
        // No previous state should allow update
        let temp_file = NamedTempFile::new().unwrap();
        let manager = StateManager::new(Some(temp_file.path().to_path_buf())).unwrap();
        
        let (should, reason) = manager.should_update(
            "example.com",
            true,   // IP changed
            false,  // not forced
            Some(30),    // min_interval
            Some(86400), // max_interval
            Some(300),   // min_error_interval
        );
        
        assert!(should);
        assert!(reason.is_none());
    }

    #[test]
    fn test_should_update_force() {
        // Force flag should always allow update
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = StateManager::new(Some(temp_file.path().to_path_buf())).unwrap();
        
        // Set recent successful update
        let state = manager.get_mut("example.com");
        state.update_success(
            IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)),
            "good".to_string()
        );
        
        let (should, reason) = manager.should_update(
            "example.com",
            true,   // IP changed
            true,   // FORCED
            Some(3600), // min_interval (1 hour)
            Some(86400), // max_interval
            Some(300),   // min_error_interval
        );
        
        assert!(should);
        assert!(reason.is_none());
    }

    #[test]
    fn test_should_update_min_interval_blocks() {
        // Recent successful update within min-interval should block
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = StateManager::new(Some(temp_file.path().to_path_buf())).unwrap();
        
        // Set recent successful update (just now)
        let state = manager.get_mut("example.com");
        state.update_success(
            IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)),
            "good".to_string()
        );
        
        let (should, reason) = manager.should_update(
            "example.com",
            true,   // IP changed
            false,  // not forced
            Some(3600), // min_interval (1 hour - definitely not passed)
            Some(86400), // max_interval
            Some(300),   // min_error_interval
        );
        
        assert!(!should);
        assert!(reason.is_some());
        assert!(reason.unwrap().contains("min-interval"));
    }

    #[test]
    fn test_should_update_no_ip_change() {
        // No IP change should skip update
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = StateManager::new(Some(temp_file.path().to_path_buf())).unwrap();
        
        let state = manager.get_mut("example.com");
        state.update_success(
            IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)),
            "good".to_string()
        );
        
        let (should, reason) = manager.should_update(
            "example.com",
            false,  // IP NOT changed
            false,  // not forced
            Some(30),    // min_interval
            Some(86400), // max_interval
            Some(300),   // min_error_interval
        );
        
        assert!(!should);
        assert!(reason.is_some());
        assert!(reason.unwrap().contains("hasn't changed"));
    }

    #[test]
    fn test_should_update_min_error_interval_blocks() {
        // Recent failed update within min-error-interval should block
        let temp_file = NamedTempFile::new().unwrap();
        let mut manager = StateManager::new(Some(temp_file.path().to_path_buf())).unwrap();
        
        // Set recent failed update
        let state = manager.get_mut("example.com");
        state.update_failure("Connection timeout".to_string());
        
        let (should, reason) = manager.should_update(
            "example.com",
            true,   // IP changed
            false,  // not forced
            Some(30),    // min_interval
            Some(86400), // max_interval
            Some(3600),  // min_error_interval (1 hour - definitely not passed)
        );
        
        assert!(!should);
        assert!(reason.is_some());
        assert!(reason.unwrap().contains("min-error-interval"));
    }
}
