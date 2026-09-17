//! Sliding-window rate limiter.
//!
//! Two policies:
//!   1. **Per-IP**: at most `per_ip_limit` drips within a rolling 1-hour window.
//!   2. **Per-recipient**: at most one drip per `per_address_cooldown` per EVM address.
//!
//! Both are in-memory + tokio-Mutex protected. Sufficient for a single faucet
//! instance fronted by a sticky load balancer; swap for a Redis-backed
//! impl when you horizontally scale.

use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use alloy_primitives::Address;
use tokio::sync::Mutex;

const IP_WINDOW: Duration = Duration::from_secs(60 * 60); // 1 hour

#[derive(Debug, Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<Inner>>,
    per_ip_limit: u32,
    per_address_cooldown: Duration,
}

#[derive(Debug, Default)]
struct Inner {
    ip_history: HashMap<IpAddr, Vec<Instant>>,
    address_last: HashMap<Address, Instant>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum LimiterError {
    #[error("ip rate limit exceeded: {hits} drips in last hour (max {max})")]
    IpExceeded { hits: u32, max: u32 },
    #[error("address cooldown active: try again in {seconds_remaining}s")]
    AddressCooldown { seconds_remaining: u64 },
}

impl RateLimiter {
    pub fn new(per_ip_limit: u32, per_address_cooldown: Duration) -> Self {
        Self { inner: Arc::new(Mutex::new(Inner::default())), per_ip_limit, per_address_cooldown }
    }

    /// Try to register a drip from `ip` to `address`. Atomic: either records
    /// both or rejects both.
    pub async fn check_and_record(&self, ip: IpAddr, address: Address) -> Result<(), LimiterError> {
        let now = Instant::now();
        let mut guard = self.inner.lock().await;

        // Per-IP window.
        let history = guard.ip_history.entry(ip).or_default();
        history.retain(|t| now.duration_since(*t) <= IP_WINDOW);
        let hits = history.len() as u32;
        if hits >= self.per_ip_limit {
            return Err(LimiterError::IpExceeded { hits, max: self.per_ip_limit });
        }

        // Per-recipient cooldown.
        if let Some(last) = guard.address_last.get(&address).copied() {
            let elapsed = now.duration_since(last);
            if elapsed < self.per_address_cooldown {
                let remaining = self.per_address_cooldown - elapsed;
                return Err(LimiterError::AddressCooldown {
                    seconds_remaining: remaining.as_secs(),
                });
            }
        }

        // Commit both.
        guard.ip_history.entry(ip).or_default().push(now);
        guard.address_last.insert(address, now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn addr_n(n: u8) -> Address {
        Address::from([n; 20])
    }

    fn ip(s: &str) -> IpAddr {
        IpAddr::from_str(s).unwrap()
    }

    #[tokio::test]
    async fn within_ip_limit_allowed() {
        let l = RateLimiter::new(3, Duration::from_secs(1));
        // Distinct recipient addresses so the per-address cooldown doesn't trigger.
        for i in 100u8..103 {
            l.check_and_record(ip("1.1.1.1"), addr_n(i)).await.unwrap();
        }
    }

    #[tokio::test]
    async fn ip_limit_blocks() {
        let l = RateLimiter::new(2, Duration::from_secs(0));
        l.check_and_record(ip("2.2.2.2"), addr_n(1)).await.unwrap();
        l.check_and_record(ip("2.2.2.2"), addr_n(2)).await.unwrap();
        let err = l.check_and_record(ip("2.2.2.2"), addr_n(3)).await.unwrap_err();
        assert!(matches!(err, LimiterError::IpExceeded { .. }));
    }

    #[tokio::test]
    async fn address_cooldown_blocks() {
        let l = RateLimiter::new(100, Duration::from_secs(60));
        l.check_and_record(ip("3.3.3.3"), addr_n(9)).await.unwrap();
        let err = l.check_and_record(ip("4.4.4.4"), addr_n(9)).await.unwrap_err();
        assert!(matches!(err, LimiterError::AddressCooldown { .. }));
    }

    #[tokio::test]
    async fn rejected_drip_does_not_commit_ip_hit() {
        // If address cooldown rejects, the IP hit count should NOT advance —
        // otherwise a hostile address could starve a legitimate IP.
        let l = RateLimiter::new(2, Duration::from_secs(60));
        l.check_and_record(ip("5.5.5.5"), addr_n(10)).await.unwrap();
        for _ in 0..5 {
            let _ = l.check_and_record(ip("5.5.5.5"), addr_n(10)).await;
        }
        // 5.5.5.5 should still have 1 drip left.
        l.check_and_record(ip("5.5.5.5"), addr_n(11)).await.unwrap();
    }
}
