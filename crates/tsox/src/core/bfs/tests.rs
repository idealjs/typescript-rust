use super::*;
use std::sync::Arc;

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
