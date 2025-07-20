//! # Rate Limiting
//!
//! This module provides rate limiting functionality to prevent abuse.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;

use crate::security::{RateLimitConfig, SecurityError, SecurityResult};

/// Rate limit store
static RATE_LIMIT_STORE: Lazy<Arc<RwLock<HashMap<String, RateLimitEntry>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Rate limit entry
#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
    last_request: Instant,
}

/// Initialize rate limiting
pub fn initialize(_config: &RateLimitConfig) -> SecurityResult<()> {
    println!("🚦 Rate limiting initialized");
    Ok(())
}

/// Check if request is within rate limit
pub fn check_rate_limit(key: &str, config: &RateLimitConfig) -> SecurityResult<()> {
    let now = Instant::now();
    let window_duration = Duration::from_secs(60); // 1 minute window
    
    let mut store = RATE_LIMIT_STORE.write().unwrap();
    
    let entry = store.entry(key.to_string()).or_insert(RateLimitEntry {
        count: 0,
        window_start: now,
        last_request: now,
    });
    
    // Reset window if expired
    if now.duration_since(entry.window_start) >= window_duration {
        entry.count = 0;
        entry.window_start = now;
    }
    
    // Check rate limit
    if entry.count >= config.requests_per_minute {
        return Err(SecurityError::RateLimitExceeded);
    }
    
    // Update counters
    entry.count += 1;
    entry.last_request = now;
    
    Ok(())
}
