use anyhow::Result;
use std::time::Duration;

pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
        }
    }
}

/// Retry a blocking operation with exponential backoff and jitter.
pub async fn with_retry<F, T>(config: &RetryConfig, op_name: &str, f: F) -> Result<T>
where
    F: Fn() -> Result<T>,
{
    let mut last_err = None;

    for attempt in 0..=config.max_retries {
        match f() {
            Ok(val) => return Ok(val),
            Err(e) => {
                if attempt == config.max_retries {
                    last_err = Some(e);
                    break;
                }

                let delay_ms = std::cmp::min(
                    config.base_delay_ms * 2u64.pow(attempt),
                    config.max_delay_ms,
                );
                // Add 0-25% jitter
                let jitter = (delay_ms as f64 * rand_jitter()) as u64;
                let total_delay = delay_ms + jitter;

                log::warn!(
                    "{} failed (attempt {}/{}): {}. Retrying in {}ms...",
                    op_name,
                    attempt + 1,
                    config.max_retries + 1,
                    e,
                    total_delay
                );

                tokio::time::sleep(Duration::from_millis(total_delay)).await;
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("{} failed after all retries", op_name)))
}

/// Simple deterministic jitter (0.0 to 0.25) using timestamp-based seed.
/// Not cryptographic, just good enough for backoff jitter.
fn rand_jitter() -> f64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos % 250) as f64 / 1000.0
}
