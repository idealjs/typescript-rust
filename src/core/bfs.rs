use std::collections::HashSet;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Default)]
pub struct SyncSet<K: Eq + Hash + Clone> {
    inner: Mutex<HashSet<K>>,
}

impl<K: Eq + Hash + Clone> SyncSet<K> {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashSet::new()),
        }
    }

    pub fn add_if_absent(&self, key: &K) -> bool {
        self.inner.lock().unwrap().insert(key.clone())
    }

    pub fn has(&self, key: &K) -> bool {
        self.inner.lock().unwrap().contains(key)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_bfs() {

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
                (false, false)
            },
        );

        assert!(!result.stopped, "Expected search to not stop early");
        assert!(
            result.path.is_empty(),
            "Expected empty path when visit function never returns true"
        );

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

        let graph = diamond_graph();
        let result = bfs_parallel(
            "A".to_string(),
            move |n| graph.get(n).cloned().unwrap_or_default(),
            |n| match n.as_str() {
                "A" => (true, false),
                "D" => (true, true),
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
    fn bfs_parallel_early_termination() {

        let mut graph = std::collections::HashMap::new();
        graph.insert(
            "Root".to_string(),
            vec!["L1A".to_string(), "L1B".to_string()],
        );
        graph.insert(
            "L1A".to_string(),
            vec!["L2A".to_string(), "L2B".to_string()],
        );
        graph.insert("L1B".to_string(), vec!["L2C".to_string()]);
        graph.insert("L2A".to_string(), vec!["L3A".to_string()]);
        graph.insert("L2B".to_string(), vec![]);
        graph.insert("L2C".to_string(), vec![]);
        graph.insert("L3A".to_string(), vec![]);

        let visited: Arc<SyncSet<String>> = Arc::new(SyncSet::new());
        let visited_for_search = visited.clone();
        bfs_parallel_ex(
            "Root".to_string(),
            move |n| graph.get(n).cloned().unwrap_or_default(),
            |n| (n == "L2B", true),
            |_| (),
            visited_for_search,
            |n| n.clone(),
        );

        assert!(visited.has(&"Root".to_string()), "Expected to visit Root");
        assert!(visited.has(&"L1A".to_string()), "Expected to visit L1A");
        assert!(visited.has(&"L1B".to_string()), "Expected to visit L1B");
        assert!(visited.has(&"L2A".to_string()), "Expected to visit L2A");
        assert!(visited.has(&"L2B".to_string()), "Expected to visit L2B");

        assert!(
            !visited.has(&"L3A".to_string()),
            "Expected not to visit L3A"
        );
    }

    #[test]
    fn bfs_parallel_returns_fallback() {

        let graph = diamond_graph();
        let visited: Arc<SyncSet<String>> = Arc::new(SyncSet::new());
        let visited_for_search = visited.clone();
        let result = bfs_parallel_ex(
            "A".to_string(),
            move |n| graph.get(n).cloned().unwrap_or_default(),
            |n| (n == "A", false),
            |_| (),
            visited_for_search,
            |n| n.clone(),
        );

        assert!(!result.stopped, "Expected search to not stop early");
        assert_eq!(result.path, vec!["A".to_string()]);
        assert!(visited.has(&"B".to_string()), "Expected to visit B");
        assert!(visited.has(&"C".to_string()), "Expected to visit C");
        assert!(visited.has(&"D".to_string()), "Expected to visit D");
    }
}
