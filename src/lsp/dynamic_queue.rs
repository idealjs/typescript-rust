//! Cancellable dynamic queue (1:1 port of Go's `internal/lsp/dynamic_queue.go`).
//!
//! Inspired by Brian C. Mills' "Rethinking Classical Concurrency Patterns".
//! This queue is a state machine where each state is a channel, "idle" or
//! "ready". The `get` function waits until the "ready" channel holds the
//! state. Putting an item grabs the state from any channel, modifies it,
//! and puts it back.

#![allow(dead_code)]

use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

struct DynamicQueueState<T> {
    items: Vec<T>,
}

/// A cancellable dynamic queue.
///
/// Go: `type dynamicQueue[T any] struct { ... }`.
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
        // Start in the idle state.
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

    /// Put an item into the queue. Returns `Err` if cancelled (by sending
    /// `None` to the cancel signal).
    pub fn put(&self, item: T) -> Result<(), T> {
        // Try to get state from either idle or ready channel.
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

        // If no immediate state, block on both.
        if state.is_none() {
            // Simple approach: block on idle first, then ready.
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

    /// Get an item from the queue. Blocks until an item is available.
    /// Returns `None` if the queue is cancelled.
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

    /// Tries to get an item without blocking. Returns `None` if no item
    /// is immediately available.
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
