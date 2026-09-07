use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn parallel() {
    let wg = new_work_group(false);
    let counter = Arc::new(AtomicUsize::new(0));
    for _ in 0..10 {
        let counter = counter.clone();
        wg.queue(Box::new(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        }));
    }
    wg.run_and_wait();
    assert_eq!(counter.load(Ordering::SeqCst), 10);
}

#[test]
fn single_threaded() {
    let wg = new_work_group(true);
    let counter = Arc::new(AtomicUsize::new(0));
    for _ in 0..10 {
        let counter = counter.clone();
        wg.queue(Box::new(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        }));
    }
    wg.run_and_wait();
    assert_eq!(counter.load(Ordering::SeqCst), 10);
}
