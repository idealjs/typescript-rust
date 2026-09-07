use super::*;
use std::time::Duration;

#[test]
fn tracer_basic() {
    let tracer = Tracer::enabled();
    assert!(tracer.is_enabled());

    {
        let _span = tracer.start("test");
        std::thread::sleep(Duration::from_micros(100));
    }

    let events = tracer.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].name, "test");
    assert!(events[0].duration >= 50);
}

#[test]
fn tracer_disabled() {
    let tracer = Tracer::new();
    assert!(!tracer.is_enabled());

    {
        let _span = tracer.start("test");
    }

    let events = tracer.events();
    assert!(events.is_empty());
}

#[test]
fn tracer_record_type() {
    let tracer = Tracer::enabled();
    tracer.record_type(1, vec!["String".to_string()], "IntrinsicType");
    tracer.record_type(2, vec!["Number".to_string()], "IntrinsicType");

    let recordings = tracer.type_recordings();
    assert_eq!(recordings.len(), 2);
    assert_eq!(recordings[0].type_id, 1);
    assert_eq!(recordings[1].type_id, 2);
}
