use anthill_di::Injection;
use tokio::{sync::{oneshot::{Receiver, Sender}}};
use tokio::sync::oneshot;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Run,
    Stop,
}

pub struct SystemState {
    state: State,
    senders_collection: Option<Vec<Sender<()>>>,
}

impl Injection for SystemState {
    fn build_injection(_: &mut anthill_di::Injector) -> Result<Self, anthill_di::DiError> {
        Ok(Self{state: State::Run, senders_collection: Some(Vec::new())})
    }
}

impl SystemState {
    pub fn stop(&mut self) -> State {
        self.state = State::Stop;

        if let Some(senders_collection) = self.senders_collection.take() {
            for sender in senders_collection.into_iter() {
                sender.send(()).unwrap();
            }
        }

        self.state
    }

    pub fn start(&mut self) -> State {
        self.state = State::Run;
        self.state
    }

    pub fn get_state(&self) -> State {
        self.state
    }

    pub fn wait_stop(&mut self) -> Receiver<()> {
        let (sender, receiver) = oneshot::channel();

        if let Some(senders_collection) = &mut self.senders_collection {
            senders_collection.push(sender);
        } else {
            self.senders_collection = Some(vec![sender]);
        }
        
        receiver
    }
}