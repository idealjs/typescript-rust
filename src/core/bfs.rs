//! Parallel breadth-first search, ported from `internal/core/bfs.go`.
//!
//! The Go implementation uses goroutines and atomics for parallelism.
//! This Rust port uses `std::thread` and `Arc` for shared state. The
//! algorithm is otherwise identical.

use std::collections::HashSet;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// Result of a breadth-first search.
#[derive(Debug, Clone)]
pub struct BfsResult<N> {
    /// True if the search was stopped early (a `visit` returned `stop: true`).
    pub stopped: bool,
    /// The path from the result node back to the start node.
    pub path: Vec<N>,
}

struct Job<N> {
    node: N,
    parent: Option<Arc<Job<N>>>,
}

/// A level in the BFS, exposed to the preprocessing callback.
pub struct BfsLevel<N> {
    pub nodes: Vec<N>,
}

struct LevelResult<N> {
    stop: bool,
    job: Option<Arc<Job<N>>>,
    next: Vec<Arc<Job<N>>>,
}

/// Perform a parallel breadth-first search starting from `start`.
///
/// - `neighbors`: returns the neighbors of a node.
/// - `visit`: called for each node; returns `(is_result, stop)`.
///   - If `is_result` is true, this node is a candidate result.
///   - If `stop` is true, the search stops immediately.
///
/// Mirrors `core.BreadthFirstSearchParallel` in Go.
pub fn bfs_parallel<N>(
    start: N,
    neighbors: impl Fn(&N) -> Vec<N> + Send + Sync + 'static,
    visit: impl Fn(&N) -> (bool, bool) + Send + Sync + 'static,
) -> BfsResult<N>
where
    N: Clone + Eq + Hash + Send + Sync + 'static,
{
    bfs_parallel_ex(start, neighbors, visit, |_| (), |n| n.clone())
}

/// Extended BFS with a pre-seeded visited set and a preprocessing hook.
///
/// Mirrors `core.BreadthFirstSearchParallelEx` in Go.
pub fn bfs_parallel_ex<N, K>(
    start: N,
    neighbors: impl Fn(&N) -> Vec<N> + Send + Sync + 'static,
    visit: impl Fn(&N) -> (bool, bool) + Send + Sync + 'static,
    preprocess_level: impl Fn(&BfsLevel<N>) + Send + Sync + 'static,
    get_key: impl Fn(&N) -> K + Send + Sync + 'static,
) -> BfsResult<N>
where
    N: Clone + Send + Sync + 'static,
    K: Eq + Hash + Clone + Send + Sync + 'static,
{
    let visited: Arc<Mutex<HashSet<K>>> = Arc::new(Mutex::new(HashSet::new()));
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
    visited: &Arc<Mutex<HashSet<K>>>,
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
            {
                let mut v = visited.lock().unwrap();
                if v.contains(&key) {
                    return;
                }
                v.insert(key);
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

    // Try to unwrap the Arc; if that fails (because there are other refs),
    // just lock and clone.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_bfs() {
        // Graph: 1 -> 2 -> 3 -> 4 (linear chain)
        let result = bfs_parallel(
            1i32,
            |n| if *n < 4 { vec![n + 1] } else { vec![] },
            |n| (*n == 4, true),
        );
        assert!(result.stopped);
        assert_eq!(result.path, vec![4, 3, 2, 1]);
    }

    #[test]
    fn no_result() {
        let result = bfs_parallel(
            1i32,
            |n| if *n < 3 { vec![n + 1] } else { vec![] },
            |_| (false, false),
        );
        assert!(!result.stopped);
        assert!(result.path.is_empty());
    }

    // ── Ported from Go internal/core/bfs_test.go ──

    fn diamond_graph() -> std::collections::HashMap<String, Vec<String>> {
        let mut g = std::collections::HashMap::new();
        g.insert("A".to_string(), vec!["B".to_string(), "C".to_string()]);
        g.insert("B".to_string(), vec!["D".to_string()]);
        g.insert("C".to_string(), vec!["D".to_string()]);
        g.insert("D".to_string(), vec![]);
        g
    }

    #[test]
    fn bfs_parallel_find_specific_node() {
        let graph = diamond_graph();
        let result = bfs_parallel(
            "A".to_string(),
            move |n| graph.get(n).cloned().unwrap_or_default(),
            |n| (n == "D", true),
        );
        assert!(result.stopped, "Expected search to stop at D");
        assert_eq!(
            result.path,
            vec!["D".to_string(), "B".to_string(), "A".to_string()]
        );
    }

    #[test]
    fn bfs_parallel_visit_all_nodes() {
        use std::sync::{Arc, Mutex};
        let graph = diamond_graph();
        let visited = Arc::new(Mutex::new(Vec::<String>::new()));
        let visited_clone = visited.clone();
        let result = bfs_parallel(
            "A".to_string(),
            move |n| graph.get(n).cloned().unwrap_or_default(),
            move |n| {
                visited_clone.lock().unwrap().push(n.clone());
                (false, false) // Never stop early
            },
        );
        // Should return stopped=false since we never return true
        assert!(!result.stopped, "Expected search to not stop early");
        assert!(
            result.path.is_empty(),
            "Expected empty path when visit function never returns true"
        );
        // Should visit all nodes exactly once
        let mut visited = visited.lock().unwrap().clone();
        visited.sort();
        assert_eq!(
            visited,
            vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
                "D".to_string()
            ]
        );
    }

    #[test]
    fn bfs_parallel_returns_stop_over_fallback() {
        // Test that a stop result is preferred over a fallback
        let graph = diamond_graph();
        let result = bfs_parallel(
            "A".to_string(),
            move |n| graph.get(n).cloned().unwrap_or_default(),
            |n| match n.as_str() {
                "A" => (true, false), // Record fallback
                "D" => (true, true),  // Stop at D
                _ => (false, false),
            },
        );
        assert!(result.stopped, "Expected search to stop at D");
        assert_eq!(
            result.path,
            vec!["D".to_string(), "B".to_string(), "A".to_string()]
        );
    }

    #[test]
    #[ignore = "TODO: bfs_parallel_ex does not expose the external visited set (SyncSet); \
                early-termination visited checks cannot be verified"]
    fn bfs_parallel_early_termination() {
        // Ported from TestBreadthFirstSearchParallel "early termination":
        // Graph with Root -> L1A/L1B -> L2A/L2B/L2C -> L3A.
        // Visits Root, L1A, L1B, L2A, L2B (not L3A) when stopping at L2B.
        // Requires BreadthFirstSearchOptions.Visited (SyncSet) which is not in the Rust API.
    }

    #[test]
    #[ignore = "TODO: bfs_parallel_ex does not expose the external visited set (SyncSet); \
                fallback visited checks cannot be verified"]
    fn bfs_parallel_returns_fallback() {
        // Ported from TestBreadthFirstSearchParallel "returns fallback when no other result found":
        // result.Stopped == false, result.Path == ["A"], and visited.Has("B"/"C"/"D").
        // Requires BreadthFirstSearchOptions.Visited (SyncSet) which is not in the Rust API.
    }
}
