use super::*;

#[test]
fn parse_watch_file_kind_roundtrip() {
    assert_eq!(
        parse_watch_file_kind("UseFsEvents"),
        Some(WatchFileKind::UseFsEvents)
    );
    assert_eq!(
        parse_watch_file_kind("fixedpollinginterval"),
        Some(WatchFileKind::FixedPollingInterval)
    );
    assert_eq!(parse_watch_file_kind("bogus"), None);
}

#[test]
fn parse_watch_directory_kind_roundtrip() {
    assert_eq!(
        parse_watch_directory_kind("UseFsEvents"),
        Some(WatchDirectoryKind::UseFsEvents)
    );
    assert_eq!(
        parse_watch_directory_kind("dynamicprioritypolling"),
        Some(WatchDirectoryKind::DynamicPriorityPolling)
    );
    assert_eq!(parse_watch_directory_kind("bogus"), None);
}

#[test]
fn parse_polling_kind_roundtrip() {
    assert_eq!(
        parse_polling_kind("FixedInterval"),
        Some(PollingKind::FixedInterval)
    );
    assert_eq!(
        parse_polling_kind("priorityinterval"),
        Some(PollingKind::PriorityInterval)
    );
    assert_eq!(parse_polling_kind("bogus"), None);
}

#[test]
fn default_is_empty_and_interval() {
    let w = WatchOptions::default();
    assert!(w.is_empty());
    assert_eq!(w.watch_interval_ms(), 2000);
}

#[test]
fn non_default_is_not_empty() {
    let w = WatchOptions {
        interval: Some(100),
        ..WatchOptions::default()
    };
    assert!(!w.is_empty());
    assert_eq!(w.watch_interval_ms(), 100);
}
