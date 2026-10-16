use std::sync::atomic::{AtomicUsize, Ordering};

use crate::guard::Deadline;

pub struct PoolStats {
    pub completed: usize,
    pub pending: usize,
}

pub fn run_pool<F>(jobs: usize, total: usize, deadline: &Deadline, task: F) -> PoolStats
where
    F: Fn(usize) + Sync + Send,
{
    let next = AtomicUsize::new(0);
    let completed = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..jobs {
            scope.spawn(|| loop {
                if deadline.exceeded() {
                    break;
                }
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= total {
                    break;
                }
                task(i);
                completed.fetch_add(1, Ordering::Relaxed);
            });
        }
    });
    let completed = completed.into_inner();
    PoolStats {
        completed,
        pending: total - completed,
    }
}
