// Address management module for handling multiple server URLs
// This module provides load balancing and failover capabilities

use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Single address context with availability state and TTL
#[derive(Debug)]
pub struct AddressContext {
    /// Server URL
    url: String,
    /// Whether the address is currently available
    available: Arc<RwLock<bool>>,
    /// Time-to-live duration for this address
    ttl: Duration,
    /// Creation time for TTL calculation
    created_at: Instant,
}

impl AddressContext {
    /// Create a new address context
    pub fn new(url: String, ttl: Duration) -> Self {
        Self {
            url,
            available: Arc::new(RwLock::new(true)),
            ttl,
            created_at: Instant::now(),
        }
    }

    /// Get the URL
    pub fn url(&self) -> String {
        self.url.clone()
    }

    /// Check if the address is available
    pub fn is_available(&self) -> bool {
        *self.available.read()
    }

    /// Mark address as unavailable
    pub fn mark_unavailable(&self) {
        *self.available.write() = false;
    }

    /// Mark address as available
    pub fn mark_available(&self) {
        *self.available.write() = true;
    }

    /// Check if the address context has expired based on TTL
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// Address manager for managing multiple server addresses
/// Provides random load balancing and availability tracking
#[derive(Debug)]
pub struct AddressManager {
    /// List of address contexts
    addresses: Arc<RwLock<Vec<Arc<AddressContext>>>>,
}

impl AddressManager {
    /// Create a new address manager with static address list
    pub fn new(urls: Vec<String>) -> Self {
        let addresses = urls
            .into_iter()
            .map(|url| {
                // Use a long TTL for static addresses (24 hours)
                Arc::new(AddressContext::new(url, Duration::from_secs(86400)))
            })
            .collect();

        Self {
            addresses: Arc::new(RwLock::new(addresses)),
        }
    }

    /// Get the number of addresses
    pub fn address_count(&self) -> usize {
        self.addresses.read().len()
    }

    /// Get a random available address
    /// Returns None if no addresses are available
    pub fn get_random_address(&self) -> Option<Arc<AddressContext>> {
        let addresses = self.addresses.read();

        // Filter available and non-expired addresses
        let available: Vec<_> = addresses
            .iter()
            .filter(|ctx| ctx.is_available() && !ctx.is_expired())
            .cloned()
            .collect();

        if available.is_empty() {
            return None;
        }

        // Use random index for load balancing
        let index = rand::random::<usize>() % available.len();
        Some(available[index].clone())
    }

    /// Mark an address as unavailable
    pub fn mark_unavailable(&self, url: &str) {
        let addresses = self.addresses.read();
        for addr in addresses.iter() {
            if addr.url() == url {
                addr.mark_unavailable();
                break;
            }
        }
    }

    /// Mark an address as available
    pub fn mark_available(&self, url: &str) {
        let addresses = self.addresses.read();
        for addr in addresses.iter() {
            if addr.url() == url {
                addr.mark_available();
                break;
            }
        }
    }

    /// Start background refresh task for dynamic address discovery
    /// TODO: This will be implemented in the next task for dynamic address discovery
    #[allow(dead_code)]
    pub async fn start_refresh_task(&self, _refresh_interval: Duration) {
        // Placeholder for dynamic address refresh from Artemis server
        // Will be implemented when adding dynamic service discovery
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_address_manager() {
        // Create manager with static address list
        let addresses = vec![
            "http://node1:8080".to_string(),
            "http://node2:8080".to_string(),
            "http://node3:8080".to_string(),
        ];
        let manager = AddressManager::new(addresses.clone());

        // Verify addresses are loaded
        assert_eq!(manager.address_count(), 3);

        // Get random address multiple times (should return valid addresses)
        for _ in 0..10 {
            let addr = manager.get_random_address();
            assert!(addr.is_some());
            let url = addr.unwrap().url();
            assert!(addresses.contains(&url));
        }
    }

    #[test]
    fn test_address_marking() {
        // Create manager with 3 addresses
        let addresses = vec![
            "http://node1:8080".to_string(),
            "http://node2:8080".to_string(),
            "http://node3:8080".to_string(),
        ];
        let manager = AddressManager::new(addresses.clone());

        // Mark node2 as unavailable
        manager.mark_unavailable("http://node2:8080");

        // Get random address multiple times (should NOT return node2)
        for _ in 0..20 {
            let addr = manager.get_random_address();
            assert!(addr.is_some());
            let url = addr.unwrap().url();
            assert_ne!(url, "http://node2:8080");
            assert!(url == "http://node1:8080" || url == "http://node3:8080");
        }

        // Mark node2 as available again
        manager.mark_available("http://node2:8080");

        // Now node2 should be returned again
        let mut found_node2 = false;
        for _ in 0..30 {
            if let Some(addr) = manager.get_random_address() {
                if addr.url() == "http://node2:8080" {
                    found_node2 = true;
                    break;
                }
            }
        }
        assert!(found_node2, "node2 should be available again");
    }

    #[test]
    fn test_address_context() {
        use std::time::Duration;

        // Create address context with 10 second TTL
        let context = AddressContext::new("http://test:8080".to_string(), Duration::from_secs(10));

        // Initially available
        assert!(context.is_available());
        assert_eq!(context.url(), "http://test:8080");

        // Mark as unavailable
        context.mark_unavailable();
        assert!(!context.is_available());

        // Mark as available
        context.mark_available();
        assert!(context.is_available());

        // Check TTL expiration
        assert!(!context.is_expired(), "Should not be expired immediately");

        // Create expired context (0 second TTL)
        let expired_context = AddressContext::new("http://test:8080".to_string(), Duration::from_secs(0));
        std::thread::sleep(Duration::from_millis(10));
        assert!(expired_context.is_expired(), "Should be expired after 10ms with 0s TTL");
    }

    #[test]
    fn test_all_addresses_unavailable() {
        // Create manager with 2 addresses
        let addresses = vec![
            "http://node1:8080".to_string(),
            "http://node2:8080".to_string(),
        ];
        let manager = AddressManager::new(addresses);

        // Mark all as unavailable
        manager.mark_unavailable("http://node1:8080");
        manager.mark_unavailable("http://node2:8080");

        // Should return None when all addresses are unavailable
        assert!(manager.get_random_address().is_none());
    }
}
