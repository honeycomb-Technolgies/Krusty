//! Rate limiting and retry logic
//!
//! Provides exponential backoff with jitter for handling API rate limits and transient errors.
//!
//! Used by subagent API calls to handle transient errors like rate limiting (429)
//! and server errors (500, 502, 503, 504).

mod backoff;

pub use backoff::{is_retryable_status, with_retry, IsRetryable, RetryConfig};
