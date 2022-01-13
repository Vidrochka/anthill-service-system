use std::{collections::LinkedList, sync::Arc};

use anthill_di::{Constructor, DependencyContext, types::BuildDependencyResult};
use async_trait::async_trait;
use tokio::{sync::{oneshot::{Receiver, Sender, channel}, RwLock}};

struct CancellationState {
    cancelled: bool,
    observers: LinkedList<Sender<()>>,
}

impl CancellationState {
    fn new() -> Self {
        Self {
            cancelled: false,
            observers: LinkedList::new(),
        }
    }
}

pub struct CancellationHandler {
    state: Arc<RwLock<CancellationState>>,
}

#[async_trait]
impl Constructor for CancellationHandler {
    async fn ctor(_ctx: DependencyContext) -> BuildDependencyResult<Self> { Ok(Self::new()) }
}

impl CancellationHandler {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(CancellationState::new())),
        }
    }

    pub async fn is_cancelled(&self) -> bool {
        self.state.read().await.cancelled
    }

    pub async fn cancel(&mut self) {
        let state_clone = self.state.clone();
        
        tokio::spawn(async move {
            let mut state_guard = state_clone.write().await;
            state_guard.cancelled = true;

            while let Some(observer) = state_guard.observers.pop_front() {
                observer.send(()).unwrap()
            }
        });
    }

    pub async fn get_cancel_waiter(&mut self) -> Receiver<()> {
        let (sender, receiver) = channel();

        let mut state_guard = self.state.write().await;
        if state_guard.cancelled {
            sender.send(()).unwrap();
        } else {
            state_guard.observers.push_back(sender);
        }

        receiver
    }
}

impl Clone for CancellationHandler {
    fn clone(&self) -> Self {
        Self { state: self.state.clone(), }
    }
}