use super::*;

#[test]
fn insertion_order() {
    let mut m = OrderedMap::new();
    m.insert("b", 2);
    m.insert("a", 1);
    m.insert("c", 3);
    let keys: Vec<&str> = m.keys().copied().collect();
    assert_eq!(keys, vec!["b", "a", "c"]);
}

#[test]
fn update_preserves_order() {
    let mut m = OrderedMap::new();
    m.insert("a", 1);
    m.insert("b", 2);
    m.insert("a", 10);
    let keys: Vec<&str> = m.keys().copied().collect();
    assert_eq!(keys, vec!["a", "b"]);
    assert_eq!(m.get(&"a"), Some(&10));
}

#[test]
fn remove_preserves_order() {
    let mut m = OrderedMap::new();
    m.insert("a", 1);
    m.insert("b", 2);
    m.insert("c", 3);
    m.remove(&"b");
    let keys: Vec<&str> = m.keys().copied().collect();
    assert_eq!(keys, vec!["a", "c"]);
}

#[test]
fn diff() {
    let mut m1 = OrderedMap::new();
    m1.insert("a", 1);
    m1.insert("b", 2);
    m1.insert("c", 3);
    let mut m2 = OrderedMap::new();
    m2.insert("a", 1);
    m2.insert("b", 20);
    m2.insert("d", 4);

    let mut added: Vec<(String, i32)> = Vec::new();
    let mut removed: Vec<(String, i32)> = Vec::new();
    let mut modified: Vec<(String, i32, i32)> = Vec::new();
    diff_ordered_maps(
        &m1,
        &m2,
        |a, b| a == b,
        |k, v| added.push((k.to_string(), *v)),
        |k, v| removed.push((k.to_string(), *v)),
        |k, v1, v2| modified.push((k.to_string(), *v1, *v2)),
    );
    assert_eq!(added, vec![("d".to_string(), 4)]);
    assert_eq!(removed, vec![("c".to_string(), 3)]);
    assert_eq!(modified, vec![("b".to_string(), 2, 20)]);
}

fn pad_int(n: i32) -> String {
    format!("{:>10}", n)
}

#[test]
fn test_ordered_map() {
    let mut m: OrderedMap<i32, String> = OrderedMap::new();

    assert!(!m.has(&1));

    const N: i32 = 1000;
    const START: i32 = 1;
    const END: i32 = START + N;

    for i in START..END {
        m.set(i, pad_int(i));
    }

    assert_eq!(m.len(), N as usize);

    for i in (START..END).rev() {
        m.set(i, pad_int(i));
    }

    assert_eq!(m.len(), N as usize);

    for i in START..END {
        let v = m.get(&i);
        assert!(v.is_some());
        assert_eq!(v.unwrap(), &pad_int(i));
    }

    for (k, v) in m.iter() {
        assert_eq!(v, &pad_int(*k));
    }

    let keys: Vec<i32> = m.keys().copied().collect();
    assert_eq!(keys.len(), N as usize);
    assert!(keys.windows(2).all(|w| w[0] <= w[1]));

    let values: Vec<String> = m.values().cloned().collect();
    assert_eq!(values.len(), N as usize);
    assert!(values.windows(2).all(|w| w[0] <= w[1]));

    let first_key = *m.keys().next().unwrap();
    assert_eq!(first_key, START);

    let first_value = m.values().next().unwrap().clone();
    assert_eq!(first_value, pad_int(START));

    let (fk, fv) = m.iter().next().unwrap();
    assert_eq!(*fk, START);
    assert_eq!(*fv, pad_int(START));

    for i in (START + 1)..END {
        let v = m.delete(&i);
        assert!(v.is_some());
        assert_eq!(v.unwrap(), pad_int(i));
        assert!(!m.has(&i));

        assert!(m.get(&i).is_none());

        assert!(m.delete(&i).is_none());
    }

    assert_eq!(m.len(), 1);
    assert!(m.has(&START));

    let v = m.delete(&START);
    assert!(v.is_some());
    assert_eq!(v.unwrap(), pad_int(START));

    assert_eq!(m.len(), 0);
}

#[test]
fn test_ordered_map_clone() {
    let mut m: OrderedMap<i32, String> = OrderedMap::new();
    m.set(1, "one".to_string());
    m.set(2, "two".to_string());

    let clone = m.clone();

    assert_eq!(clone.len(), 2);
    let clone_keys: Vec<i32> = clone.keys().copied().collect();
    assert_eq!(clone_keys, vec![1, 2]);
    let clone_values: Vec<String> = clone.values().cloned().collect();
    assert_eq!(clone_values, vec!["one".to_string(), "two".to_string()]);

    let v = clone.get(&1);
    assert!(v.is_some());
    assert_eq!(v.unwrap(), "one");

    m.delete(&1);

    assert_eq!(m.len(), 1);
    assert_eq!(clone.len(), 2);
    let clone_keys: Vec<i32> = clone.keys().copied().collect();
    assert_eq!(clone_keys, vec![1, 2]);
    let clone_values: Vec<String> = clone.values().cloned().collect();
    assert_eq!(clone_values, vec!["one".to_string(), "two".to_string()]);
}

#[test]
fn test_ordered_map_clear() {
    let mut m: OrderedMap<i32, String> = OrderedMap::new();
    m.set(1, "one".to_string());
    m.set(2, "two".to_string());

    m.clear();

    assert_eq!(m.len(), 0);
}

#[test]
fn test_ordered_map_with_size_hint() {
    const N: usize = 1024;
    let mut m = OrderedMap::with_capacity(N);
    for i in 0..N {
        m.set(i, i);
    }

    assert_eq!(m.len(), N);

    for i in 0..N {
        assert_eq!(m.get(&i), Some(&i));
    }

    let keys: Vec<usize> = m.keys().copied().collect();
    assert_eq!(keys, (0..N).collect::<Vec<_>>());
}

#[test]
fn test_ordered_map_unmarshal_json() {
    let m: OrderedMap<String, serde_json::Value> =
        serde_json::from_str(r#"{"a": 1, "b": "two", "c": { "d": 4 } }"#).unwrap();
    assert_eq!(m.len(), 3);
    assert_eq!(m.get(&"a".to_string()).and_then(|v| v.as_f64()), Some(1.0));
    let keys: Vec<String> = m.keys().cloned().collect();
    assert_eq!(
        keys,
        vec!["a".to_string(), "b".to_string(), "c".to_string()]
    );

    let m: OrderedMap<String, serde_json::Value> = serde_json::from_str("null").unwrap();
    assert_eq!(m.len(), 0);

    let err = serde_json::from_str::<OrderedMap<String, serde_json::Value>>(r#""foo""#);
    assert!(err.is_err());

    let err = serde_json::from_str::<OrderedMap<i32, serde_json::Value>>(r#"{"a": 1, "b": "two"}"#);
    assert!(err.is_err());
}
