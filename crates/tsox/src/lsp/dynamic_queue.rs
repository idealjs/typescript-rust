#![allow(dead_code)]

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

struct DynamicQueueState<T> {
    items: Vec<T>,
}

pub struct DynamicQueue<T: Send> {
    idle_tx: Sender<Option<DynamicQueueState<T>>>,
    idle_rx: Receiver<Option<DynamicQueueState<T>>>,
    ready_tx: Sender<Option<DynamicQueueState<T>>>,
    ready_rx: Receiver<Option<DynamicQueueState<T>>>,
}

impl<T: Send> DynamicQueue<T> {
    pub fn new() -> Arc<Self> {
        let (idle_tx, idle_rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel();

        idle_tx
            .send(Some(DynamicQueueState { items: Vec::new() }))
            .ok();
        Arc::new(DynamicQueue {
            idle_tx,
            idle_rx,
            ready_tx,
            ready_rx,
        })
    }

    pub fn put(&self, item: T) -> Result<(), T> {
        let mut state = self
            .idle_rx
            .recv_timeout(Duration::from_secs(0))
            .ok()
            .flatten()
            .or_else(|| {
                self.ready_rx
                    .recv_timeout(Duration::from_secs(0))
                    .ok()
                    .flatten()
            });

        if state.is_none() {
            state = self.idle_rx.recv().ok().flatten();
            if state.is_none() {
                state = self.ready_rx.recv().ok().flatten();
            }
        }

        match state {
            None => Err(item),
            Some(mut s) => {
                s.items.push(item);
                self.ready_tx.send(Some(s)).ok();
                Ok(())
            }
        }
    }

    pub fn get(&self) -> Option<T> {
        let mut state = self.ready_rx.recv().ok().flatten()?;
        if state.items.is_empty() {
            self.idle_tx.send(Some(state)).ok();
            return None;
        }
        let item = state.items.remove(0);
        if state.items.is_empty() {
            self.idle_tx.send(Some(state)).ok();
        } else {
            self.ready_tx.send(Some(state)).ok();
        }
        Some(item)
    }

    pub fn try_get(&self) -> Option<T> {
        let mut state = self.ready_rx.recv_timeout(Duration::from_secs(0)).ok()??;
        if state.items.is_empty() {
            self.idle_tx.send(Some(state)).ok();
            return None;
        }
        let item = state.items.remove(0);
        if state.items.is_empty() {
            self.idle_tx.send(Some(state)).ok();
        } else {
            self.ready_tx.send(Some(state)).ok();
        }
        Some(item)
    }
}
