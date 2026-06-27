use std::time::{Duration, Instant};

pub struct RateLimiter {
    single_doi_interval: Duration,
    list_query_interval: Duration,
    last_single_doi: Option<Instant>,
    last_list_query: Option<Instant>,
    backoff_until: Option<Instant>,
    backoff_multiplier: u32,
}

impl RateLimiter {
    pub fn new() -> Self {
        RateLimiter {
            single_doi_interval: Duration::from_millis(100),
            list_query_interval: Duration::from_millis(334),
            last_single_doi: None,
            last_list_query: None,
            backoff_until: None,
            backoff_multiplier: 0,
        }
    }

    pub fn can_do_single_doi(&self) -> bool {
        if let Some(until) = self.backoff_until
            && Instant::now() < until
        {
            return false;
        }
        if let Some(last) = self.last_single_doi {
            return Instant::now().duration_since(last) >= self.single_doi_interval;
        }
        true
    }

    pub fn can_do_list_query(&self) -> bool {
        if let Some(until) = self.backoff_until
            && Instant::now() < until
        {
            return false;
        }
        if let Some(last) = self.last_list_query {
            return Instant::now().duration_since(last) >= self.list_query_interval;
        }
        true
    }

    pub fn record_single_doi(&mut self) {
        self.last_single_doi = Some(Instant::now());
    }

    pub fn record_list_query(&mut self) {
        self.last_list_query = Some(Instant::now());
    }

    pub fn trigger_backoff(&mut self) {
        let delay = Duration::from_secs(1u64 << self.backoff_multiplier.min(6));
        self.backoff_until = Some(Instant::now() + delay);
        self.backoff_multiplier = (self.backoff_multiplier + 1).min(6);
    }

    pub fn reset_backoff(&mut self) {
        self.backoff_multiplier = 0;
        self.backoff_until = None;
    }

    pub fn wait_duration(&self) -> Duration {
        if let Some(until) = self.backoff_until {
            let now = Instant::now();
            if now < until {
                return until - now;
            }
        }
        Duration::ZERO
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}
