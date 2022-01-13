use std::time::Duration;

use anthill_di::{Constructor, types::BuildDependencyResult, DependencyContext};
use async_trait::async_trait;

pub struct CoreConfig {
    pub on_start_timeout: Duration,
    pub on_stop_timeout: Duration,
}

#[async_trait]
impl Constructor for CoreConfig {
    async fn ctor(_ctx: DependencyContext) -> BuildDependencyResult<Self> {
        Ok(Self {
            on_start_timeout: Duration::from_millis(5000),
            on_stop_timeout: Duration::from_millis(5000),
        })
    }
}