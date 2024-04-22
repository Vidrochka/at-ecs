use std::collections::HashMap;

use async_broadcast::RecvError;
use async_lock::RwLock;
use uuid::Uuid;


#[derive(Debug)]
pub struct Signal<T: Clone> {
    pub sender: async_broadcast::Sender<T>,
    pub reciever: async_broadcast::Receiver<T>,
}

impl<T: Clone> Signal<T> {
    pub fn new() -> Self {
        let (sender, reciever) = async_broadcast::broadcast(1);
        Self { sender, reciever }
    }
    
    pub async fn signal(&self, signal: T) {
        _ = self.sender.broadcast(signal).await;
    }

    pub fn signal_waiter(&self) -> SignalWaiter<T> {
        SignalWaiter(self.reciever.clone())
    }
}

#[derive(Debug)]
pub struct SignalWaiter<T: Clone>(async_broadcast::Receiver<T>);

#[derive(Debug, thiserror::Error)]
pub enum WaitError {
    #[error("Channal closed")]
    Closed,
    #[error(transparent)]
    Caston(anyhow::Error),
}

impl<T: Clone> SignalWaiter<T> {
    pub async fn wait(&mut self) -> Result<T, WaitError> {
        self.0.recv().await
            .map_err(|e| match e {
                RecvError::Closed => WaitError::Closed,
                e => WaitError::Caston(e.into())
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SingalWaiterId(Uuid);

#[derive(Debug)]
pub struct SignalWaitersCollection<T: Clone> {
    waiters: RwLock<HashMap<SingalWaiterId, SignalWaiter<T>>>,
}

impl<T: Clone> SignalWaitersCollection<T> {
    pub fn new() -> Self {
        Self { waiters: Default::default() }
    }

    pub async fn subscribe(&self, w: SignalWaiter<T>) -> SingalWaiterId {
        let id = SingalWaiterId(Uuid::now_v7());
        self.waiters.write().await.insert(id, w);
        id
    }

    pub async fn subscribe_many(&self, ws: Vec<SignalWaiter<T>>) -> Vec<SingalWaiterId> {
        let mut ids = Vec::new();
        
        let mut waiters_lock = self.waiters.write().await;

        for w in ws {
            let id = SingalWaiterId(Uuid::now_v7());
            waiters_lock.insert(id, w);
            ids.push(id);
        }

        ids
    }

    pub async fn unsubscribe(&self, id: &SingalWaiterId) {
        self.waiters.write().await.remove(&id);
    }

    pub async fn wait_all(&self) -> Vec<Result<T, WaitError>> {
        let mut results = Vec::new();

        for waiter in self.waiters.write().await.values_mut() {
            let res = waiter.wait().await;
            results.push(res);
        }

        results
    }
}