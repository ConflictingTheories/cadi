use crate::error::{Error, Result};
use crate::types::ScraperConfig;
use reqwest::Client;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// HTTP fetcher with rate limiting and caching
pub struct Fetcher {
    client: Client,
    config: ScraperConfig,
    rate_limiter: Arc<Mutex<RateLimiter>>,
}

/// Rate limiter using token bucket algorithm
struct RateLimiter {
    tokens: f64,
    capacity: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl RateLimiter {
    fn new(rate_per_second: f64) -> Self {
        Self {
            tokens: rate_per_second,
            capacity: rate_per_second,
            refill_rate: rate_per_second,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let new_tokens = elapsed * self.refill_rate;
        self.tokens = (self.tokens + new_tokens).min(self.capacity);
        self.last_refill = now;
    }

    fn acquire(&mut self, tokens: f64) -> Duration {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            Duration::from_secs(0)
        } else {
            let wait_time = (tokens - self.tokens) / self.refill_rate;
            Duration::from_secs_f64(wait_time)
        }
    }
}

impl Fetcher {
    /// Create a new fetcher with the given configuration
    pub fn new(config: ScraperConfig) -> Result<Self> {
        let timeout = Duration::from_secs(config.request_timeout);
        let client = Client::builder()
            .timeout(timeout)
            .gzip(true)
            .build()?;

        let rate_limiter = Arc::new(Mutex::new(RateLimiter::new(config.rate_limit)));

        Ok(Self {
            client,
            config,
            rate_limiter,
        })
    }

    /// Fetch content from a URL
    pub async fn fetch_url(&self, url: &str) -> Result<Vec<u8>> {
        let mut limiter = self.rate_limiter.lock().await;
        let wait_time = limiter.acquire(1.0);
        drop(limiter);

        tokio::time::sleep(wait_time).await;

        tracing::info!("Fetching URL: {}", url);

        let response = self
            .client
            .get(url)
            .header("User-Agent", "CADI-Scraper/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Fetch(format!(
                "HTTP {} from {}",
                response.status(),
                url
            )));
        }

        let bytes = response.bytes().await?.to_vec();
        tracing::debug!("Fetched {} bytes from {}", bytes.len(), url);

        Ok(bytes)
    }

    /// Fetch content from a local file
    pub async fn fetch_file(&self, path: &Path) -> Result<Vec<u8>> {
        tracing::info!("Reading file: {}", path.display());

        let content = tokio::fs::read(path).await?;
        tracing::debug!("Read {} bytes from {}", content.len(), path.display());

        Ok(content)
    }

    /// Recursively fetch all files in a directory
    pub async fn fetch_directory(&self, path: &Path, patterns: Option<&[String]>) -> Result<Vec<(PathBuf, Vec<u8>)>> {
        use walkdir::WalkDir;
        use glob::Pattern;

        let mut results = Vec::new();
        let exclude = self.config.exclude_patterns.clone();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            let relative = file_path
                .strip_prefix(path)
                .unwrap_or(file_path)
                .to_path_buf();

            // Check exclude patterns
            let path_str = relative.to_string_lossy();
            if exclude.iter().any(|p| {
                Pattern::new(p)
                    .ok()
                    .and_then(|pattern| pattern.matches(&path_str).then_some(true))
                    .unwrap_or(false)
            }) {
                continue;
            }

            // Check include patterns if specified
            if let Some(patterns) = patterns {
                if !patterns.iter().any(|p| {
                    Pattern::new(p)
                        .ok()
                        .and_then(|pattern| pattern.matches(&path_str).then_some(true))
                        .unwrap_or(false)
                }) {
                    continue;
                }
            }

            if let Ok(content) = self.fetch_file(file_path).await {
                results.push((relative, content));
            }
        }

        tracing::info!("Fetched {} files from directory", results.len());
        Ok(results)
    }

    /// Clone the fetcher for concurrent use
    pub fn clone_for_concurrent(&self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            client: self.client.clone(),
            config: self.config.clone(),
            rate_limiter: Arc::clone(&self.rate_limiter),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(10.0);
        let wait = limiter.acquire(5.0);
        assert_eq!(wait, Duration::from_secs(0));
        assert_eq!(limiter.tokens, 5.0);
    }
}
