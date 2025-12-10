# Provider Implementation Guidelines

This document provides guidelines for implementing new DNS provider clients in rddclient.

## Overview

rddclient uses a trait-based architecture where each DNS provider implements the `DnsClient` trait. This ensures consistent behavior across all providers while allowing provider-specific implementations.

## The DnsClient Trait

Located in `src/clients/mod.rs`, the `DnsClient` trait defines the interface all providers must implement:

```rust
pub trait DnsClient {
    /// Update a DNS record with the given hostname and IP address
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>>;
    
    /// Validate the client configuration
    fn validate_config(&self) -> Result<(), Box<dyn Error>>;
    
    /// Get the provider name for logging
    fn provider_name(&self) -> &str;
}
```

## Step-by-Step Implementation

### 1. Create Provider Module

Create a new file in `src/clients/` named after your provider (e.g., `newprovider.rs`):

```rust
use crate::clients::DnsClient;
use crate::config::Config;
use std::error::Error;
use std::net::IpAddr;

/// NewProvider DNS client
/// Brief description of the provider and its API
pub struct NewProviderClient {
    server: String,
    api_token: String,
    // Add provider-specific fields
}

impl NewProviderClient {
    pub fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        // Extract required configuration
        let api_token = config.password.as_ref()
            .ok_or("API token (password) is required for NewProvider")?
            .clone();
        
        // Set default server if not provided
        let server = config.server.clone()
            .unwrap_or_else(|| "https://api.newprovider.com".to_string());
        
        Ok(Self {
            server,
            api_token,
        })
    }
}

impl DnsClient for NewProviderClient {
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>> {
        // Determine record type based on IP version
        let record_type = match ip {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };
        
        log::info!("Updating {} with NewProvider", hostname);
        
        // Make API call to update DNS record
        // Example structure - adjust for your provider's API
        let url = format!("{}/dns/records/{}", self.server, hostname);
        let body = serde_json::json!({
            "type": record_type,
            "value": ip.to_string(),
        });
        
        let response = minreq::post(&url)
            .with_header("Authorization", format!("Bearer {}", self.api_token))
            .with_header("User-Agent", crate::USER_AGENT)
            .with_json(&body)?
            .send()?;
        
        if response.status_code >= 200 && response.status_code < 300 {
            log::info!("Successfully updated {} to {}", hostname, ip);
            Ok(())
        } else {
            Err(format!("HTTP {} error: {}", response.status_code, 
                       response.as_str()?).into())
        }
    }
    
    fn validate_config(&self) -> Result<(), Box<dyn Error>> {
        if self.api_token.is_empty() {
            return Err("API token is required for NewProvider".into());
        }
        Ok(())
    }
    
    fn provider_name(&self) -> &str {
        "NewProvider"
    }
}
```

### 2. Register Provider in mod.rs

Add your provider to `src/clients/mod.rs`:

```rust
// Add module declaration
pub mod newprovider;

// Add to create_client() function
pub fn create_client(provider: &str, config: &Config) -> Result<Box<dyn DnsClient>, Box<dyn Error>> {
    match provider.to_lowercase().as_str() {
        // ... existing providers ...
        "newprovider" | "new-provider" => Ok(Box::new(newprovider::NewProviderClient::new(config)?)),
        // ... rest of providers ...
    }
}
```

Also add to the error message list of supported providers.

### 3. Add Tests

Add unit tests at the bottom of your provider module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_config() -> Config {
        Config {
            protocol: Some("newprovider".to_string()),
            password: Some("test_token".to_string()),
            server: Some("https://api.newprovider.com".to_string()),
            ..Default::default()
        }
    }
    
    #[test]
    fn test_newprovider_client_creation() {
        let config = create_test_config();
        let client = NewProviderClient::new(&config);
        assert!(client.is_ok());
    }
    
    #[test]
    fn test_newprovider_missing_token() {
        let mut config = create_test_config();
        config.password = None;
        let client = NewProviderClient::new(&config);
        assert!(client.is_err());
    }
    
    #[test]
    fn test_newprovider_validate_config() {
        let config = create_test_config();
        let client = NewProviderClient::new(&config).unwrap();
        assert!(client.validate_config().is_ok());
    }
    
    #[test]
    fn test_newprovider_provider_name() {
        let config = create_test_config();
        let client = NewProviderClient::new(&config).unwrap();
        assert_eq!(client.provider_name(), "NewProvider");
    }
}
```

### 4. Create Example Configuration

Add an example config file in `examples/` (e.g., `examples/newprovider.conf`):

```ini
# NewProvider Dynamic DNS Configuration
# Get your API token from https://newprovider.com/account/api

protocol=newprovider
password=your_api_token_here
host=ddns.example.com
```

### 5. Update Documentation

Update the README.md to include your provider in the supported providers list.

## API Implementation Guidelines

### Authentication Methods

Different providers use different authentication methods. Common patterns:

**Bearer Token:**
```rust
.with_header("Authorization", format!("Bearer {}", self.api_token))
```

**API Key Header:**
```rust
.with_header("X-API-Key", &self.api_key)
```

**Basic Authentication:**
```rust
use base64::{Engine as _, engine::general_purpose};
let auth = format!("{}:{}", self.username, self.password);
let encoded = general_purpose::STANDARD.encode(auth.as_bytes());
.with_header("Authorization", format!("Basic {}", encoded))
```

**Query Parameters:**
```rust
let url = format!("{}?token={}&hostname={}&ip={}", 
                  self.server, self.token, hostname, ip);
```

### Error Handling

Always provide clear error messages:

```rust
// Parse API error responses
if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
    if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
        return Err(format!("Provider API error: {}", error).into());
    }
}

// Generic HTTP error
Err(format!("HTTP {} error: {}", status_code, body).into())
```

### Response Parsing

Common response patterns:

**JSON API:**
```rust
let response = minreq::post(&url)
    .with_json(&body)?
    .send()?;

let json: serde_json::Value = response.json()?;
if json.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
    Ok(())
} else {
    Err("Update failed".into())
}
```

**Plain Text (DynDNS2 protocol):**
```rust
let body = response.as_str()?.trim();

if body.starts_with("good") || body.starts_with("nochg") {
    log::info!("Successfully updated");
    Ok(())
} else if body.starts_with("badauth") {
    Err("Authentication failed".into())
} else {
    Err(format!("Unexpected response: {}", body).into())
}
```

## Best Practices

### 1. IPv6 Support

Always support both IPv4 and IPv6:

```rust
let record_type = match ip {
    IpAddr::V4(_) => "A",
    IpAddr::V6(_) => "AAAA",
};
```

### 2. Logging

Use appropriate log levels:

```rust
log::info!("Updating {} with Provider", hostname);  // User-facing actions
log::debug!("API response: {}", body);              // Debug information
log::error!("Failed to update: {}", error);         // Errors
```

### 3. User-Agent Header

Always include the user agent:

```rust
.with_header("User-Agent", crate::USER_AGENT)
```

### 4. Configuration Validation

Validate all required fields in `new()`:

```rust
let api_key = config.password.as_ref()
    .ok_or("API key is required")?
    .clone();

if api_key.is_empty() {
    return Err("API key cannot be empty".into());
}
```

### 5. Default Values

Provide sensible defaults where appropriate:

```rust
let server = config.server.clone()
    .unwrap_or_else(|| "https://api.provider.com".to_string());

let ttl = config.ttl.unwrap_or(300);
```

### 6. Subdomain Extraction

If your provider needs the domain extracted from a FQDN:

```rust
fn extract_subdomain(&self, hostname: &str) -> String {
    if hostname == self.zone {
        return "@".to_string();
    }
    
    if let Some(subdomain) = hostname.strip_suffix(&format!(".{}", self.zone)) {
        subdomain.to_string()
    } else {
        hostname.to_string()
    }
}
```

## Testing

### Required Tests

1. **Client Creation**: Test successful client instantiation
2. **Missing Credentials**: Test error handling for missing required fields
3. **Validation**: Test `validate_config()` method
4. **Provider Name**: Verify `provider_name()` returns correct value
5. **Edge Cases**: Test single-label hostnames, special characters, etc.

### Running Tests

```bash
# Run all tests
cargo test

# Run specific provider tests
cargo test newprovider

# Run with output
cargo test -- --nocapture
```

## Common Patterns by Provider Type

### REST API (Cloudflare, DigitalOcean, etc.)

1. Find zone/domain ID via API
2. Find record ID for hostname
3. Update record via PUT/PATCH request
4. Parse JSON response

### DynDNS2 Protocol (Many providers)

1. Construct URL with query parameters
2. Send GET request with Basic Auth
3. Parse plain text response (good/nochg/badauth/etc.)

### Simple Token-Based (DuckDNS, Freedns, etc.)

1. Construct URL with token and IP in query string
2. Send GET request
3. Check for "OK" or success indicator in response

## Security Considerations

1. **Never log credentials**: Don't log passwords, tokens, or API keys
2. **Use HTTPS**: Default to HTTPS URLs, allow HTTP only if explicitly configured
3. **Validate inputs**: Sanitize hostnames and other user inputs
4. **Secure random generation**: Use `rand::thread_rng()` for any random data (like salts)
5. **Basic Auth in headers**: Don't put credentials in URL parameters when possible

## Provider-Specific Notes

### Rate Limiting

Some providers have rate limits. Consider:
- Checking current record before updating
- Using cache/state management (handled by rddclient core)
- Documenting rate limits in provider comments

### TTL Support

If the provider supports custom TTL:

```rust
let ttl = config.ttl.unwrap_or(300);
```

### Multiple Record Support

Some providers support updating multiple records in one call. This can be added as an optimization but isn't required.

## Documentation Requirements

Each provider should have:

1. **Module-level doc comment**: Brief description and API reference link
2. **Configuration example**: In `examples/` directory
3. **README entry**: In the supported providers list
4. **Provider-specific notes**: Any quirks, requirements, or limitations

## Getting Help

- Check existing provider implementations for reference
- See `src/clients/cloudflare.rs` for a full REST API example
- See `src/clients/dyndns2.rs` for DynDNS2 protocol example
- See `src/clients/duckdns.rs` for simple token-based example

## Checklist for New Providers

- [ ] Create `src/clients/newprovider.rs` with `DnsClient` implementation
- [ ] Add module declaration to `src/clients/mod.rs`
- [ ] Add to `create_client()` factory function
- [ ] Add to error message provider list
- [ ] Write at least 4 unit tests
- [ ] Create example config in `examples/`
- [ ] Update README.md supported providers list
- [ ] Support both IPv4 and IPv6
- [ ] Include proper error handling
- [ ] Add logging statements
- [ ] Validate configuration in `new()`
- [ ] Test with real credentials (if possible)
- [ ] Document any provider-specific requirements

## Validation Against ddclient

When implementing a provider that exists in ddclient:

1. Check `ddclient/ddclient.in` for the reference implementation
2. Find the `nic_PROVIDER_update` function
3. Match the authentication method
4. Match the API endpoint and parameters
5. Match the response parsing logic
6. Test against the same provider to ensure compatibility

## Example Providers to Reference

- **Cloudflare** (`src/clients/cloudflare.rs`): Full REST API with zone/record lookup
- **DynDNS2** (`src/clients/dyndns2.rs`): Generic DynDNS2 protocol implementation
- **DuckDNS** (`src/clients/duckdns.rs`): Simple token-based GET request
- **DigitalOcean** (`src/clients/digitalocean.rs`): REST API with pagination
- **NFSN** (`src/clients/nfsn.rs`): Custom authentication (SHA1-based)
- **Dinahosting** (`src/clients/dinahosting.rs`): Basic Auth with domain extraction
