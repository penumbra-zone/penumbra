/// Configuration for the summoner.
#[derive(Clone, Copy)]
pub struct Config {
    pub phase1_timeout_secs: u64,
    pub phase2_timeout_secs: u64,
    pub min_bid_u64: u64,
    pub max_strikes: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            phase1_timeout_secs: 12 * 60,
            phase2_timeout_secs: 8 * 60,
            min_bid_u64: 1,
            max_strikes: 3,
        }
    }
}

impl Config {
    pub fn with_phase1_timeout_secs(mut self, x: Option<u64>) -> Self {
        if let Some(x) = x {
            self.phase1_timeout_secs = x;
        }
        self
    }

    pub fn with_phase2_timeout_secs(mut self, x: Option<u64>) -> Self {
        if let Some(x) = x {
            self.phase2_timeout_secs = x;
        }
        self
    }

    pub fn with_min_bid_u64(mut self, x: Option<u64>) -> Self {
        if let Some(x) = x {
            self.min_bid_u64 = x;
        }
        self
    }

    pub fn with_max_strikes(mut self, x: Option<u64>) -> Self {
        if let Some(x) = x {
            self.max_strikes = x;
        }
        self
    }
}
