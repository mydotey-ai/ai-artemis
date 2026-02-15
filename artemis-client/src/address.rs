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

    /// Get the HTTP URL
    pub fn http_url(&self) -> &str {
        &self.url
    }

    /// Get the WebSocket URL for a given path
    /// Converts http:// to ws:// and https:// to wss://
    pub fn ws_url(&self, path: &str) -> String {
        let ws_url = self.url.replace("http://", "ws://").replace("https://", "wss://");
        format!("{}{}", ws_url, path)
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
    /// Static addresses have a long TTL (24 hours) and are not refreshed
    pub fn new_static(urls: Vec<String>) -> Self {
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

    /// Create a new address manager with dynamic address list
    /// Dynamic addresses can be updated and have configurable TTL
    pub fn new_dynamic(initial_urls: Vec<String>, address_ttl: Duration) -> Self {
        let addresses = initial_urls
            .into_iter()
            .map(|url| Arc::new(AddressContext::new(url, address_ttl)))
            .collect();

        Self {
            addresses: Arc::new(RwLock::new(addresses)),
        }
    }

    /// Get the number of addresses
    pub fn address_count(&self) -> usize {
        self.addresses.read().len()
    }

    /// Get a random available address as a String
    /// Returns None if no addresses are available
    /// Filters out unavailable and expired addresses
    pub async fn get_random_address(&self) -> Option<String> {
        let addresses = self.addresses.read();

        if addresses.is_empty() {
            return None;
        }

        // Filter available and non-expired addresses
        let available: Vec<_> = addresses
            .iter()
            .filter(|ctx| ctx.is_available() && !ctx.is_expired())
            .collect();

        if available.is_empty() {
            // Fallback to first address if all are unavailable
            tracing::warn!("No available addresses, using fallback");
            return Some(addresses[0].http_url().to_string());
        }

        // Use random index for load balancing
        let index = rand::random::<usize>() % available.len();
        Some(available[index].http_url().to_string())
    }

    /// Get all addresses as a Vec<String>
    pub async fn get_all_addresses(&self) -> Vec<String> {
        self.addresses
            .read()
            .iter()
            .map(|ctx| ctx.http_url().to_string())
            .collect()
    }

    /// Mark an address as unavailable
    pub async fn mark_unavailable(&self, url: &str) {
        let addresses = self.addresses.read();
        for addr in addresses.iter() {
            if addr.http_url() == url {
                addr.mark_unavailable();
                tracing::debug!("Marked address as unavailable: {}", url);
                break;
            }
        }
    }

    /// Mark an address as available
    pub async fn mark_available(&self, url: &str) {
        let addresses = self.addresses.read();
        for addr in addresses.iter() {
            if addr.http_url() == url {
                addr.mark_available();
                tracing::debug!("Marked address as available: {}", url);
                break;
            }
        }
    }

    /// Update the entire address list
    /// Useful for dynamic address discovery
    pub async fn update_addresses(&self, new_urls: Vec<String>) {
        // Get TTL from existing addresses, or use default
        let ttl = self.addresses.read()
            .first()
            .map(|ctx| ctx.ttl)
            .unwrap_or(Duration::from_secs(3600));

        let new_addresses: Vec<Arc<AddressContext>> = new_urls
            .into_iter()
            .map(|url| Arc::new(AddressContext::new(url, ttl)))
            .collect();

        *self.addresses.write() = new_addresses;
        tracing::debug!("Updated address list with {} addresses", self.addresses.read().len());
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

    #[tokio::test]
    async fn test_static_address_manager() {
        // Create manager with static address list
        let addresses = vec![
            "http://node1:8080".to_string(),
            "http://node2:8080".to_string(),
            "http://node3:8080".to_string(),
        ];
        let manager = AddressManager::new_static(addresses.clone());

        // Verify addresses are loaded
        assert_eq!(manager.address_count(), 3);

        // Get random address multiple times (should return valid addresses)
        for _ in 0..10 {
            let addr = manager.get_random_address().await;
            assert!(addr.is_some());
            let url = addr.unwrap();
            assert!(addresses.contains(&url));
        }

        // Get all addresses
        let all = manager.get_all_addresses().await;
        assert_eq!(all.len(), 3);
        assert!(all.contains(&"http://node1:8080".to_string()));
    }

    #[tokio::test]
    async fn test_dynamic_address_manager() {
        // Create manager with dynamic address list
        let initial_urls = vec!["http://node1:8080".to_string()];
        let manager = AddressManager::new_dynamic(initial_urls, Duration::from_secs(60));

        // Initially has 1 address
        assert_eq!(manager.address_count(), 1);

        // Update to 2 addresses
        let new_urls = vec![
            "http://node1:8080".to_string(),
            "http://node2:8080".to_string(),
        ];
        manager.update_addresses(new_urls).await;

        // Now has 2 addresses
        assert_eq!(manager.address_count(), 2);
        let all = manager.get_all_addresses().await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_address_marking() {
        // Create manager with 3 addresses
        let addresses = vec![
            "http://node1:8080".to_string(),
            "http://node2:8080".to_string(),
            "http://node3:8080".to_string(),
        ];
        let manager = AddressManager::new_static(addresses.clone());

        // Mark node2 as unavailable
        manager.mark_unavailable("http://node2:8080").await;

        // Get random address multiple times (should NOT return node2)
        for _ in 0..20 {
            let addr = manager.get_random_address().await;
            assert!(addr.is_some());
            let url = addr.unwrap();
            assert_ne!(url, "http://node2:8080");
            assert!(url == "http://node1:8080" || url == "http://node3:8080");
        }

        // Mark node2 as available again
        manager.mark_available("http://node2:8080").await;

        // Now node2 should be returned again
        let mut found_node2 = false;
        for _ in 0..30 {
            if let Some(addr) = manager.get_random_address().await
                && addr == "http://node2:8080" {
                    found_node2 = true;
                    break;
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
        assert_eq!(context.http_url(), "http://test:8080");

        // Test WebSocket URL conversion
        assert_eq!(context.ws_url("/subscribe"), "ws://test:8080/subscribe");

        // Test HTTPS to WSS conversion
        let https_context = AddressContext::new("https://test:8080".to_string(), Duration::from_secs(10));
        assert_eq!(https_context.ws_url("/subscribe"), "wss://test:8080/subscribe");

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

    #[tokio::test]
    async fn test_all_addresses_unavailable() {
        // Create manager with 2 addresses
        let addresses = vec![
            "http://node1:8080".to_string(),
            "http://node2:8080".to_string(),
        ];
        let manager = AddressManager::new_static(addresses);

        // Mark all as unavailable
        manager.mark_unavailable("http://node1:8080").await;
        manager.mark_unavailable("http://node2:8080").await;

        // Should return fallback address (first address) when all are unavailable
        let addr = manager.get_random_address().await;
        assert!(addr.is_some());
        assert_eq!(addr.unwrap(), "http://node1:8080");
    }
}
