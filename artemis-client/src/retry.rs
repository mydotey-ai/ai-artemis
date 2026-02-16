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

    #[tokio::test]
    async fn test_retry_queue_is_empty() {
        let queue = RetryQueue::<String>::new(Duration::from_millis(100));

        assert!(queue.is_empty().await);

        queue.add("item".to_string()).await;
        assert!(!queue.is_empty().await);

        queue.remove(&"item".to_string()).await;
        assert!(queue.is_empty().await);
    }

    #[tokio::test]
    async fn test_retry_queue_zero_interval() {
        let queue = RetryQueue::<String>::new(Duration::from_millis(0));

        queue.add("item".to_string()).await;

        // With zero interval, item should be ready immediately
        tokio::time::sleep(Duration::from_millis(1)).await;
        let items = queue.get_items_to_retry().await;
        assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    async fn test_retry_queue_long_interval() {
        let queue = RetryQueue::<String>::new(Duration::from_secs(10));

        queue.add("item".to_string()).await;

        // With long interval, item should not be ready
        let items = queue.get_items_to_retry().await;
        assert_eq!(items.len(), 0);
    }

    #[tokio::test]
    async fn test_retry_queue_multiple_items() {
        let queue = RetryQueue::<String>::new(Duration::from_millis(50));

        queue.add("item1".to_string()).await;
        queue.add("item2".to_string()).await;
        queue.add("item3".to_string()).await;

        assert_eq!(queue.len().await, 3);

        tokio::time::sleep(Duration::from_millis(60)).await;

        let items = queue.get_items_to_retry().await;
        assert_eq!(items.len(), 3);
    }

    #[tokio::test]
    async fn test_retry_queue_remove_nonexistent() {
        let queue = RetryQueue::<String>::new(Duration::from_millis(100));

        queue.add("item1".to_string()).await;

        // Remove non-existent item
        queue.remove(&"item2".to_string()).await;

        assert_eq!(queue.len().await, 1);
    }

    #[tokio::test]
    async fn test_retry_queue_duplicate_add() {
        let queue = RetryQueue::<String>::new(Duration::from_millis(100));

        queue.add("item".to_string()).await;
        queue.add("item".to_string()).await; // Add same item again

        // Should still have only 1 item (HashMap overwrites)
        assert_eq!(queue.len().await, 1);
    }

    #[tokio::test]
    async fn test_retry_item_clone() {
        let item = RetryItem {
            data: "test".to_string(),
            last_attempt: Instant::now(),
        };

        let cloned = item.clone();
        assert_eq!(item.data, cloned.data);
    }

    #[tokio::test]
    async fn test_retry_item_debug() {
        let item = RetryItem {
            data: "test".to_string(),
            last_attempt: Instant::now(),
        };

        let debug_str = format!("{:?}", item);
        assert!(debug_str.contains("RetryItem"));
        assert!(debug_str.contains("test"));
    }

    #[tokio::test]
    async fn test_retry_queue_partial_retry() {
        let queue = RetryQueue::<String>::new(Duration::from_millis(50));

        // Add items at different times
        queue.add("item1".to_string()).await;
        tokio::time::sleep(Duration::from_millis(30)).await;
        queue.add("item2".to_string()).await;

        // Wait for first item's interval to elapse
        tokio::time::sleep(Duration::from_millis(30)).await;

        // Only first item should be ready
        let items = queue.get_items_to_retry().await;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], "item1");
    }

    #[tokio::test]
    async fn test_retry_queue_with_integers() {
        let queue = RetryQueue::<i32>::new(Duration::from_millis(50));

        queue.add(1).await;
        queue.add(2).await;
        queue.add(3).await;

        assert_eq!(queue.len().await, 3);

        tokio::time::sleep(Duration::from_millis(60)).await;

        let items = queue.get_items_to_retry().await;
        assert_eq!(items.len(), 3);
    }

    #[tokio::test]
    async fn test_retry_loop_success() {
        let queue = Arc::new(RetryQueue::<String>::new(Duration::from_millis(50)));

        queue.add("success-item".to_string()).await;

        let success_count = Arc::new(Mutex::new(0));
        let success_count_clone = success_count.clone();

        // Start retry loop with always-success function
        queue.clone().start_retry_loop(move |_item| {
            let count = success_count_clone.clone();
            async move {
                *count.lock() += 1;
                true // Always succeed
            }
        });

        // Wait for retry loop to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Item should be removed after successful retry
        assert_eq!(queue.len().await, 0);
        assert!(*success_count.lock() >= 1);
    }

    #[tokio::test]
    async fn test_retry_loop_failure() {
        let queue = Arc::new(RetryQueue::<String>::new(Duration::from_millis(50)));

        queue.add("fail-item".to_string()).await;

        let attempt_count = Arc::new(Mutex::new(0));
        let attempt_count_clone = attempt_count.clone();

        // Start retry loop with always-failure function
        queue.clone().start_retry_loop(move |_item| {
            let count = attempt_count_clone.clone();
            async move {
                *count.lock() += 1;
                false // Always fail
            }
        });

        // Wait for multiple retry attempts
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Item should still be in queue
        assert_eq!(queue.len().await, 1);
        // Should have attempted multiple times
        assert!(*attempt_count.lock() >= 2);
    }
}
