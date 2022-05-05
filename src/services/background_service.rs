use anthill_di::types::{BuildDependencyResult, BuildDependencyError};
use anthill_di::{Constructor, DependencyContext, DependencyLifeCycle};
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use super::IBaseService;

/// You cant create 'mut self' method, because while service work, execute call with read lock
#[async_trait_with_sync::async_trait(Sync)]
pub trait IBackgroundService where Self: Send + Sync + 'static {
    async fn execute(&self);
}

enum BackgroundServiceState {
    Pending,
    Started{ work_task: JoinHandle<()> },
}

pub struct BackgroundService<TService> where TService: IBackgroundService + Constructor {
    pub service: Arc<RwLock<TService>>,
    state: BackgroundServiceState,
}

#[async_trait_with_sync::async_trait(Sync)]
impl<TService> Constructor for BackgroundService<TService> where TService: IBackgroundService + Constructor {
    async fn ctor(ctx: DependencyContext) ->  BuildDependencyResult<Self> {
        ctx.register_type::<RwLock<TService>>(DependencyLifeCycle::Singleton).await
            .map_err(|e| BuildDependencyError::AddDependencyError{err: e})?
            .map_as::<RwLock<dyn IBackgroundService>>().await.map_err(|e| BuildDependencyError::Custom { message: format!("{e:?}").to_string() })?;

        Ok(Self {
            service: ctx.resolve().await?,
            state: BackgroundServiceState::Pending,
        })
    }
}

#[async_trait_with_sync::async_trait(Sync)]
impl<TService> IBaseService for BackgroundService<TService> where TService: IBackgroundService + Constructor {
    async fn on_start(&mut self) {
        let service_ref = self.service.clone();
        
        self.state = BackgroundServiceState::Started{work_task: tokio::spawn(async move {
            service_ref.read().await.execute().await;
        })}; 
    }

    async fn on_stop(&mut self) {
        if let BackgroundServiceState::Started{ work_task} = &mut self.state {
            work_task.await.unwrap();
        }
    }
}