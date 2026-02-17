use crate::error::Result;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

/// Retry an async operation with backoff
///
/// # Arguments
/// - `max_retries`: Maximum number of attempts
/// - `retry_interval`: Base interval between retries
/// - `f`: Async closure that returns Result<T>
///
/// # Returns
/// - Ok(T) on success
/// - Err with the last error if all retries exhausted
pub async fn retry_with_backoff<F, Fut, T>(
    max_retries: usize,
    retry_interval: Duration,
    mut f: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let mut last_error = None;

    for attempt in 0..max_retries {
        match f().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                if attempt < max_retries - 1 {
                    warn!(
                        "Request failed (attempt {}/{}): {}, retrying after {:?}",
                        attempt + 1,
                        max_retries,
                        e,
                        retry_interval
                    );
                    sleep(retry_interval).await;
                }
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ClientError;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        let result = retry_with_backoff(3, Duration::from_millis(10), || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok::<i32, ClientError>(42)
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        let result = retry_with_backoff(5, Duration::from_millis(10), || {
            let c = c.clone();
            async move {
                let count = c.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(ClientError::Internal("temporary failure".into()))
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        let result = retry_with_backoff(3, Duration::from_millis(10), || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err::<i32, ClientError>(ClientError::Internal("always fail".into()))
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
}
