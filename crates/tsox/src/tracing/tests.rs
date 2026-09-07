use super::*;
use std::collections::HashMap;

#[test]
fn concurrent_duration_events_use_separate_thread_ids() {
    let tr = Tracer::new();

    let end_a = tr.push(
        Phase::Parse,
        "createSourceFile",
        vec![("path".into(), TraceArg::Str("/a.ts".into()))],
    );
    let end_b = tr.push(
        Phase::Parse,
        "createSourceFile",
        vec![("path".into(), TraceArg::Str("/b.ts".into()))],
    );
    drop(end_a);
    drop(end_b);

    let end_check = tr.push(
        Phase::Check,
        "checkSourceFile",
        vec![
            ("checkerId".into(), TraceArg::Int(0)),
            ("path".into(), TraceArg::Str("/a.ts".into())),
        ],
    );
    let end_variance = tr.push(
        Phase::CheckTypes,
        "getVariancesWorker",
        vec![
            ("checkerId".into(), TraceArg::Int(0)),
            ("id".into(), TraceArg::Int(1)),
        ],
    );
    drop(end_variance);
    drop(end_check);

    let events = tr.take_events();

    let a_begin = find_event(
        &events,
        "B",
        "createSourceFile",
        "path",
        &TraceArg::Str("/a.ts".into()),
    );
    let a_end = find_event(
        &events,
        "E",
        "createSourceFile",
        "path",
        &TraceArg::Str("/a.ts".into()),
    );
    let b_begin = find_event(
        &events,
        "B",
        "createSourceFile",
        "path",
        &TraceArg::Str("/b.ts".into()),
    );
    let b_end = find_event(
        &events,
        "E",
        "createSourceFile",
        "path",
        &TraceArg::Str("/b.ts".into()),
    );
    assert_eq!(a_begin.tid, a_end.tid);
    assert_eq!(b_begin.tid, b_end.tid);
    assert_ne!(a_begin.tid, b_begin.tid);
    assert_thread_name(&events, a_begin.tid, "file:/a.ts");
    assert_thread_name(&events, b_begin.tid, "file:/b.ts");

    let check_begin = find_event(
        &events,
        "B",
        "checkSourceFile",
        "path",
        &TraceArg::Str("/a.ts".into()),
    );
    let variance_begin = find_event(&events, "B", "getVariancesWorker", "id", &TraceArg::Int(1));
    assert_eq!(check_begin.tid, variance_begin.tid);
    assert_thread_name(&events, check_begin.tid, "checker:0");

    assert_duration_events_are_well_nested_by_thread(&events);
}

#[test]
fn thread_ids_are_stable_across_first_seen_order() {
    let first = trace_thread_ids_for_paths(&["/a.ts", "/b.ts"]);
    let second = trace_thread_ids_for_paths(&["/b.ts", "/a.ts"]);
    assert_eq!(first, second);
}

fn trace_thread_ids_for_paths(paths: &[&str]) -> HashMap<String, usize> {
    let tr = Tracer::new();
    for path in paths {
        let end = tr.push(
            Phase::Parse,
            "createSourceFile",
            vec![("path".into(), TraceArg::Str((*path).into()))],
        );
        drop(end);
    }
    let events = tr.take_events();
    let mut ids = HashMap::new();
    for path in paths {
        let begin = find_event(
            &events,
            "B",
            "createSourceFile",
            "path",
            &TraceArg::Str((*path).into()),
        );
        ids.insert((*path).to_string(), begin.tid);
    }
    ids
}

fn find_event(
    events: &[TraceEvent],
    ph: &str,
    name: &str,
    arg_name: &str,
    arg_value: &TraceArg,
) -> TraceEvent {
    for event in events {
        if event.ph == ph && event.name == name {
            for (k, v) in &event.args {
                if k == arg_name && v == arg_value {
                    return event.clone();
                }
            }
        }
    }
    panic!("failed to find {ph} event {name:?} with {arg_name}={arg_value:?}");
}

fn assert_thread_name(events: &[TraceEvent], tid: usize, name: &str) {
    for event in events {
        if event.ph == "M" && event.name == "thread_name" && event.tid == tid {
            for (k, v) in &event.args {
                if k == "name" && v == &TraceArg::Str(name.to_string()) {
                    return;
                }
            }
        }
    }
    panic!("failed to find thread_name metadata for thread {tid} named {name:?}");
}

fn assert_duration_events_are_well_nested_by_thread(events: &[TraceEvent]) {
    let mut stacks: HashMap<usize, Vec<&TraceEvent>> = HashMap::new();
    for event in events {
        match event.ph {
            "B" => stacks.entry(event.tid).or_default().push(event),
            "E" => {
                let stack = stacks.entry(event.tid).or_default();
                assert!(
                    !stack.is_empty(),
                    "unmatched end event {:?} on thread {}",
                    event.name,
                    event.tid
                );
                let begin = stack.pop().unwrap();
                assert_eq!(
                    begin.cat, event.cat,
                    "mismatched cat on thread {}",
                    event.tid
                );
                assert_eq!(
                    begin.name, event.name,
                    "mismatched name on thread {}",
                    event.tid
                );
            }
            _ => {}
        }
    }
    for (tid, stack) in &stacks {
        assert!(
            stack.is_empty(),
            "thread {tid} has {} unterminated events",
            stack.len()
        );
    }
}
