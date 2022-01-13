use anthill_di::{Constructor, DependencyContext, types::BuildDependencyResult};
use async_trait::async_trait;

use crate::utils::CancellationHandler;

pub struct ApplicationLifeTime {
    pub running: CancellationHandler,
}

#[async_trait]
impl Constructor for ApplicationLifeTime {
    async fn ctor(ctx: DependencyContext) -> BuildDependencyResult<Self> {
        Ok(Self {
            running: ctx.get_transient().await?,
        })
    }
}