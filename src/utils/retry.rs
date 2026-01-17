use std::future::Future;
use crate::types::RetryConfig;

pub async fn retry_with_backoff<F, Fut, T, E>(
    retry_config: &RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut last_error = None;

    for attempt in 0..retry_config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);

                if attempt + 1 < retry_config.max_attempts {
                    let backoff = retry_config.calculate_backoff(attempt);
                    tracing::debug!(
                        "Request failed, retrying in {:?} (attempt {}/{})",
                        backoff,
                        attempt + 1,
                        retry_config.max_attempts
                    );
                    tokio::time::sleep(backoff).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}
