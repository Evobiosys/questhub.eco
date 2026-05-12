//! Spam defense layers for quest submissions.
//!
//! Layered (cheapest first):
//!   1. Honeypot field         - checked in the handler (existing)
//!   2. Rate limit per IP      - RateLimiter (this file)
//!   3. Time-trap              - issued_at_ms in the captcha token (this file)
//!   4. Proof-of-work captcha  - SHA-256 with N leading zero bits (this file)
//!
//! All state is in-memory: spam protection resets on restart, which is fine.
//! The point of these layers is to defeat automated submission, not to be a
//! durable record.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::RwLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Difficulty in leading zero BITS that the SHA-256(challenge:nonce) digest
/// must have. 18 bits ≈ 262K hashes ≈ 0.5–2 s on a modern device.
pub const POW_DIFFICULTY_BITS: u32 = 18;

/// Minimum elapsed time between challenge issue and form submission.
/// Defeats bots that submit instantly. Real users need at least this long
/// to read + fill the form.
pub const TIME_TRAP_MIN_MS: u64 = 2_000;

/// How long a challenge remains valid (also bounds memory usage).
pub const CHALLENGE_TTL: Duration = Duration::from_secs(600);

/// Rate limit: max submissions per IP per window.
pub const RATE_LIMIT_MAX: u32 = 10;
pub const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(3600);

#[derive(Clone, Debug)]
struct Challenge {
    issued_at_ms: u64,
    used: bool,
}

#[derive(Clone, Debug)]
struct RateBucket {
    window_start_ms: u64,
    count: u32,
}

pub struct SpamGuard {
    challenges: RwLock<HashMap<String, Challenge>>,
    rate_buckets: RwLock<HashMap<IpAddr, RateBucket>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CaptchaError {
    /// Challenge not found (expired, never issued, or replay).
    UnknownChallenge,
    /// PoW solution does not produce required leading zero bits.
    InvalidProof,
    /// Form submitted faster than TIME_TRAP_MIN_MS.
    TooFast,
    /// Challenge already redeemed (single-use).
    AlreadyUsed,
}

impl SpamGuard {
    pub fn new() -> Self {
        Self {
            challenges: RwLock::new(HashMap::new()),
            rate_buckets: RwLock::new(HashMap::new()),
        }
    }

    /// Issue a fresh PoW challenge. Stored with a TTL.
    pub fn issue_challenge(&self) -> (String, u32, u64) {
        let token = Uuid::new_v4().simple().to_string();
        let now_ms = now_unix_ms();
        let mut map = self.challenges.write().unwrap();
        // Opportunistic prune of expired entries.
        let cutoff = now_ms.saturating_sub(CHALLENGE_TTL.as_millis() as u64);
        map.retain(|_, c| c.issued_at_ms >= cutoff);
        map.insert(token.clone(), Challenge { issued_at_ms: now_ms, used: false });
        (token, POW_DIFFICULTY_BITS, now_ms)
    }

    /// Verify a submitted (challenge, nonce) pair. Single-use on success.
    pub fn verify(&self, challenge: &str, nonce: &str) -> Result<(), CaptchaError> {
        let mut map = self.challenges.write().unwrap();
        let entry = map.get_mut(challenge).ok_or(CaptchaError::UnknownChallenge)?;
        if entry.used {
            return Err(CaptchaError::AlreadyUsed);
        }
        let now_ms = now_unix_ms();
        if now_ms.saturating_sub(entry.issued_at_ms) < TIME_TRAP_MIN_MS {
            return Err(CaptchaError::TooFast);
        }
        if !proof_is_valid(challenge, nonce, POW_DIFFICULTY_BITS) {
            return Err(CaptchaError::InvalidProof);
        }
        entry.used = true;
        Ok(())
    }

    /// Returns true if the IP is allowed to submit; false if rate-limited.
    /// Increments the counter on success.
    pub fn check_rate_limit(&self, ip: IpAddr) -> bool {
        let now_ms = now_unix_ms();
        let window_ms = RATE_LIMIT_WINDOW.as_millis() as u64;
        let mut buckets = self.rate_buckets.write().unwrap();
        let bucket = buckets.entry(ip).or_insert(RateBucket { window_start_ms: now_ms, count: 0 });
        if now_ms.saturating_sub(bucket.window_start_ms) >= window_ms {
            bucket.window_start_ms = now_ms;
            bucket.count = 0;
        }
        if bucket.count >= RATE_LIMIT_MAX {
            return false;
        }
        bucket.count += 1;
        true
    }
}

impl Default for SpamGuard {
    fn default() -> Self { Self::new() }
}

fn now_unix_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// Check whether SHA-256("{challenge}:{nonce}") has at least `bits` leading zero bits.
fn proof_is_valid(challenge: &str, nonce: &str, bits: u32) -> bool {
    let mut h = Sha256::new();
    h.update(challenge.as_bytes());
    h.update(b":");
    h.update(nonce.as_bytes());
    let digest = h.finalize();
    let full_bytes = (bits / 8) as usize;
    let extra_bits = bits % 8;
    if digest.iter().take(full_bytes).any(|b| *b != 0) {
        return false;
    }
    if extra_bits == 0 {
        return true;
    }
    let mask = 0xFFu8 << (8 - extra_bits);
    digest[full_bytes] & mask == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proof_validation_recognises_correct_solutions() {
        // Brute force a tiny puzzle (8 bits) so the test stays fast.
        let challenge = "test-challenge";
        let mut nonce: u64 = 0;
        loop {
            let s = nonce.to_string();
            if proof_is_valid(challenge, &s, 8) {
                break;
            }
            nonce += 1;
            if nonce > 100_000 { panic!("did not find proof"); }
        }
        assert!(proof_is_valid(challenge, &nonce.to_string(), 8));
        assert!(!proof_is_valid(challenge, "0", 24));
    }

    #[test]
    fn rate_limit_blocks_after_max() {
        let g = SpamGuard::new();
        let ip: IpAddr = "1.2.3.4".parse().unwrap();
        for _ in 0..RATE_LIMIT_MAX {
            assert!(g.check_rate_limit(ip));
        }
        assert!(!g.check_rate_limit(ip));
    }
}
