use super::instance::InstanceKey;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Lease {
    key: InstanceKey,
    #[allow(dead_code)]
    creation_time: Instant,
    renewal_time: Arc<Mutex<Instant>>,
    eviction_time: Arc<Mutex<Option<Instant>>>,
    ttl: Duration,
}

impl Lease {
    pub fn new(key: InstanceKey, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            key,
            creation_time: now,
            renewal_time: Arc::new(Mutex::new(now)),
            eviction_time: Arc::new(Mutex::new(None)),
            ttl,
        }
    }

    pub fn renew(&self) {
        *self.renewal_time.lock() = Instant::now();
    }

    pub fn is_expired(&self) -> bool {
        self.renewal_time.lock().elapsed() > self.ttl
    }

    pub fn mark_evicted(&self) {
        *self.eviction_time.lock() = Some(Instant::now());
    }

    pub fn key(&self) -> &InstanceKey {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::instance::InstanceKey;
    use std::thread::sleep;

    #[test]
    fn test_lease_expiration() {
        let key = InstanceKey {
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            service_id: "service".to_string(),
            group_id: String::new(),
            instance_id: "inst".to_string(),
        };

        let lease = Lease::new(key, Duration::from_millis(100));

        assert!(!lease.is_expired());
        sleep(Duration::from_millis(150));
        assert!(lease.is_expired());
    }

    #[test]
    fn test_lease_renewal() {
        let key = InstanceKey {
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            service_id: "service".to_string(),
            group_id: String::new(),
            instance_id: "inst".to_string(),
        };

        let lease = Lease::new(key, Duration::from_millis(100));

        sleep(Duration::from_millis(60));
        lease.renew();
        sleep(Duration::from_millis(60));

        assert!(!lease.is_expired());
    }
}
