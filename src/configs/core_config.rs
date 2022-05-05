use std::time::Duration;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CoreConfig {
    #[serde(default = "default_timeout")]
    pub on_start_timeout: Duration,

    #[serde(default = "default_timeout")]
    pub on_stop_timeout: Duration,
}

fn default_timeout() -> Duration {
    Duration::from_millis(5000)
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self { on_start_timeout: Duration::from_millis(5000), on_stop_timeout: Duration::from_millis(5000) }
    }
} 