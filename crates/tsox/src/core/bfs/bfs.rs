use super::sync_set::SyncSet;
use std::collections::HashSet;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone)]
pub struct BfsResult<N> {
    pub stopped: bool,

    pub path: Vec<N>,
}

struct Job<N> {
    node: N,
    parent: Option<Arc<Job<N>>>,
}

pub struct BfsLevel<N> {
    pub nodes: Vec<N>,
}

struct LevelResult<N> {
    stop: bool,
    job: Option<Arc<Job<N>>>,
    next: Vec<Arc<Job<N>>>,
}

pub fn bfs_parallel<N>(
    start: N,
    neighbors: impl Fn(&N) -> Vec<N> + Send + Sync + 'static,
    visit: impl Fn(&N) -> (bool, bool) + Send + Sync + 'static,
) -> BfsResult<N>
where
    N: Clone + Eq + Hash + Send + Sync + 'static,
{
    let visited: Arc<SyncSet<N>> = Arc::new(SyncSet::new());
    bfs_parallel_ex(start, neighbors, visit, |_| (), visited, |n| n.clone())
}

pub fn bfs_parallel_ex<N, K>(
    start: N,
    neighbors: impl Fn(&N) -> Vec<N> + Send + Sync + 'static,
    visit: impl Fn(&N) -> (bool, bool) + Send + Sync + 'static,
    preprocess_level: impl Fn(&BfsLevel<N>) + Send + Sync + 'static,
    visited: Arc<SyncSet<K>>,
    get_key: impl Fn(&N) -> K + Send + Sync + 'static,
) -> BfsResult<N>
where
    N: Clone + Send + Sync + 'static,
    K: Eq + Hash + Clone + Send + Sync + 'static,
{
    let neighbors = Arc::new(neighbors);
    let visit = Arc::new(visit);
    let preprocess = Arc::new(preprocess_level);
    let get_key = Arc::new(get_key);

    let mut level: Vec<Arc<Job<N>>> = vec![Arc::new(Job {
        node: start,
        parent: None,
    })];

    let mut fallback: Option<Arc<Job<N>>> = None;

    while !level.is_empty() {
        let result = process_level(
            &level,
            &visited,
            &neighbors,
            &visit,
            &preprocess,
            &get_key,
            &mut fallback,
        );
        if result.stop {
            return BfsResult {
                stopped: true,
                path: create_path(&result.job.expect("stop without job")),
            };
        }
        level = result.next;
    }

    BfsResult {
        stopped: false,
        path: fallback.map(|j| create_path(&j)).unwrap_or_default(),
    }
}

#[allow(clippy::too_many_arguments)]
fn process_level<N, K>(
    level: &[Arc<Job<N>>],
    visited: &Arc<SyncSet<K>>,
    neighbors: &Arc<impl Fn(&N) -> Vec<N> + Send + Sync + 'static>,
    visit: &Arc<impl Fn(&N) -> (bool, bool) + Send + Sync + 'static>,
    preprocess: &Arc<impl Fn(&BfsLevel<N>) + Send + Sync + 'static>,
    get_key: &Arc<impl Fn(&N) -> K + Send + Sync + 'static>,
    fallback: &mut Option<Arc<Job<N>>>,
) -> LevelResult<N>
where
    N: Clone + Send + Sync + 'static,
    K: Eq + Hash + Clone + Send + Sync + 'static,
{
    let nodes: Vec<N> = level.iter().map(|j| j.node.clone()).collect();
    preprocess(&BfsLevel { nodes });

    let lowest_goal = Arc::new(AtomicUsize::new(usize::MAX));
    let lowest_fallback = Arc::new(AtomicUsize::new(usize::MAX));

    let next: Vec<Vec<Arc<Job<N>>>> = (0..level.len()).map(|_| Vec::new()).collect();
    let next = Arc::new(Mutex::new(next));

    let mut handles = Vec::new();
    for (i, job) in level.iter().cloned().enumerate() {
        let visited = visited.clone();
        let neighbors = neighbors.clone();
        let visit = visit.clone();
        let get_key = get_key.clone();
        let lowest_goal = lowest_goal.clone();
        let lowest_fallback = lowest_fallback.clone();
        let next = next.clone();

        handles.push(thread::spawn(move || {
            if i >= lowest_goal.load(Ordering::SeqCst) {
                return;
            }

            let key = get_key(&job.node);
            if !visited.add_if_absent(&key) {
                return;
            }

            let (is_result, stop) = visit(&job.node);
            if is_result {
                if stop {
                    update_min(&lowest_goal, i);
                    return;
                }
                update_min(&lowest_fallback, i);
            }

            if i >= lowest_goal.load(Ordering::SeqCst) {
                return;
            }

            let neighbor_nodes = neighbors(&job.node);
            if !neighbor_nodes.is_empty() {
                let mapped: Vec<Arc<Job<N>>> = neighbor_nodes
                    .into_iter()
                    .map(|child| {
                        Arc::new(Job {
                            node: child,
                            parent: Some(job.clone()),
                        })
                    })
                    .collect();
                next.lock().unwrap()[i] = mapped;
            }
        }));
    }

    for handle in handles {
        handle.join().expect("BFS worker panicked");
    }

    let goal_idx = lowest_goal.load(Ordering::SeqCst);
    if goal_idx != usize::MAX {
        return LevelResult {
            stop: true,
            job: Some(level[goal_idx].clone()),
            next: Vec::new(),
        };
    }

    let fallback_idx = lowest_fallback.load(Ordering::SeqCst);
    if fallback_idx != usize::MAX && fallback.is_none() {
        *fallback = Some(level[fallback_idx].clone());
    }

    let next_jobs = match Arc::try_unwrap(next) {
        Ok(mutex) => mutex.into_inner().unwrap(),
        Err(arc) => std::mem::take(&mut *arc.lock().unwrap()),
    };

    let mut result: Vec<Arc<Job<N>>> = Vec::new();
    let mut seen: HashSet<K> = HashSet::new();
    for jobs in next_jobs {
        for j in jobs {
            let key = get_key(&j.node);
            if !seen.contains(&key) {
                seen.insert(key);
                result.push(j);
            }
        }
    }

    LevelResult {
        stop: false,
        job: None,
        next: result,
    }
}

fn create_path<N: Clone>(job: &Arc<Job<N>>) -> Vec<N> {
    let mut path = Vec::new();
    let mut current = Some(job.clone());
    while let Some(j) = current {
        path.push(j.node.clone());
        current = j.parent.clone();
    }
    path
}

fn update_min(a: &AtomicUsize, candidate: usize) -> bool {
    loop {
        let current = a.load(Ordering::SeqCst);
        if current < candidate {
            return false;
        }
        if a.compare_exchange(current, candidate, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            return true;
        }
    }
}
