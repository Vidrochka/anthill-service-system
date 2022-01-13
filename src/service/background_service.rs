use std::{fmt::Debug, marker::PhantomData};

use anthill_di::{Constructor, DependencyContext, types::BuildDependencyResult};
use async_trait::async_trait;
use tokio::{task::JoinHandle};

use crate::service::HostedService;

#[async_trait]
pub trait BackgroundService where Self: Send + Sync + 'static {
    async fn execute(&mut self);
}

enum BackgroundServiceState {
    Created { service: Box<dyn BackgroundService> },
    Starting,
    Started { handler: JoinHandle<()> },
}

impl Debug for BackgroundServiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Created { .. } => f.debug_struct("Created").finish(),
            Self::Starting => write!(f, "Starting"),
            Self::Started { .. } => f.debug_struct("Started").finish(),
        }
    }
}

pub struct BackgroundServiceWrapper<T: BackgroundService> {
    state: BackgroundServiceState,
    phantom: PhantomData<T>,
}

#[async_trait]
impl<T: BackgroundService> Constructor for BackgroundServiceWrapper<T> {
    async fn ctor(ctx: DependencyContext) -> BuildDependencyResult<Self> { 
        Ok(Self{
            state: BackgroundServiceState::Created { service: Box::new(ctx.get_transient::<T>().await?) },
            phantom: PhantomData,
        })
    }
}

#[async_trait]
impl<T: BackgroundService> HostedService for BackgroundServiceWrapper<T> {
    async fn on_start(&mut self) {
        let mut state = BackgroundServiceState::Starting;
        std::mem::swap(&mut self.state, &mut state);

        if let  BackgroundServiceState::Created { mut service } = state {
            let mut handler = BackgroundServiceState::Started { 
                handler: tokio::spawn(async move {
                    service.execute().await;
                })
            };

            std::mem::swap(&mut self.state, &mut handler);

        } else {
            panic!("Неожиданное состояние сервиса {:?}", state);
        }
    }

    async fn on_stop(&mut self) {
        if let BackgroundServiceState::Started{ handler} = &mut self.state {
            handler.await.unwrap();
        }
    }
}