use parking_lot::Mutex;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// A retry item with data and last attempt timestamp
#[derive(Debug, Clone)]
struct RetryItem<T> {
    data: T,
    last_attempt: Instant,
}

/// A generic retry queue for failed operations.
///
/// Items are added after a failure and tracked with their last attempt time.
/// The `get_items_to_retry` method returns only items whose retry interval
/// has elapsed since their last attempt.
pub struct RetryQueue<T: Clone + Eq + Hash> {
    items: Arc<Mutex<HashMap<T, RetryItem<T>>>>,
    retry_interval: Duration,
}

impl<T: Clone + Eq + Hash> RetryQueue<T> {
    /// Create a new retry queue with the given retry interval
    pub fn new(retry_interval: Duration) -> Self {
        Self {
            items: Arc::new(Mutex::new(HashMap::new())),
            retry_interval,
        }
    }

    /// Add a failed item to the queue
    pub async fn add(&self, item: T) {
        let retry_item = RetryItem {
            data: item.clone(),
            last_attempt: Instant::now(),
        };
        self.items.lock().insert(item, retry_item);
        debug!("Added item to retry queue");
    }

    /// Remove an item from the queue (e.g., after successful retry)
    pub async fn remove(&self, item: &T) {
        self.items.lock().remove(item);
        debug!("Removed item from retry queue");
    }

    /// Get the number of items in the queue
    pub async fn len(&self) -> usize {
        self.items.lock().len()
    }

    /// Check if the queue is empty
    #[allow(dead_code)]
    pub async fn is_empty(&self) -> bool {
        self.items.lock().is_empty()
    }

    /// Get items ready to retry (those whose retry interval has elapsed)
    pub async fn get_items_to_retry(&self) -> Vec<T> {
        let items = self.items.lock();
        items
            .values()
            .filter(|item| item.last_attempt.elapsed() > self.retry_interval)
            .map(|item| item.data.clone())
            .collect()
    }

    /// Start a background retry loop.
    ///
    /// The `retry_fn` is called for each item ready to retry.
    /// If it returns `true`, the item is removed from the queue.
    /// If it returns `false`, the item's last attempt time is updated.
    pub fn start_retry_loop<F, Fut>(
        self: Arc<Self>,
        retry_fn: F,
    ) where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = bool> + Send,
        T: Send + Sync + 'static,
    {
        let retry_interval = self.retry_interval;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(retry_interval).await;

                let items_to_retry = self.get_items_to_retry().await;
                if items_to_retry.is_empty() {
                    continue;
                }

                info!("Retrying {} failed items", items_to_retry.len());

                for item in items_to_retry {
                    let success = retry_fn(item.clone()).await;

                    if success {
                        self.remove(&item).await;
                        info!("Retry succeeded, removed from queue");
                    } else {
                        // Update last attempt time
                        let retry_item = RetryItem {
                            data: item.clone(),
                            last_attempt: Instant::now(),
                        };
                        self.items.lock().insert(item, retry_item);
                        debug!("Retry failed, updated last attempt time");
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_queue_basic() {
        let queue = RetryQueue::<String>::new(Duration::from_millis(100));

        queue.add("item1".to_string()).await;
        queue.add("item2".to_string()).await;

        assert_eq!(queue.len().await, 2);
    }

    #[tokio::test]
    async fn test_retry_queue_retry_logic() {
        let queue = RetryQueue::<String>::new(Duration::from_millis(50));

        queue.add("item".to_string()).await;

        // Should not return items immediately (retry interval not elapsed)
        let items = queue.get_items_to_retry().await;
        assert_eq!(items.len(), 0);

        // Wait past the retry interval
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Now should return the item
        let items = queue.get_items_to_retry().await;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], "item");
    }

    #[tokio::test]
    async fn test_retry_queue_remove() {
        let queue = RetryQueue::<String>::new(Duration::from_millis(100));

        queue.add("item1".to_string()).await;
        queue.add("item2".to_string()).await;

        queue.remove(&"item1".to_string()).await;

        assert_eq!(queue.len().await, 1);
    }
}
