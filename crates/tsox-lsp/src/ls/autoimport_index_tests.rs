use crate::ls::autoimport_index::*;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TestEntry {
    name: String,
}

impl Named for TestEntry {
    fn name(&self) -> &str { ::tsox_core::fntrace::enter("name"); 
        &self.name
    }
}

#[test]
fn test_word_indices() { ::tsox_core::fntrace::enter("test_word_indices"); 
    assert_eq!(word_indices("CamelCase"), vec![0, 5]);
    assert_eq!(word_indices("snake_case"), vec![0, 6]);
    assert_eq!(word_indices("ParseURL"), vec![0, 5]);
    assert_eq!(word_indices("__proto__"), vec![0, 2]);
}

#[test]
fn test_contains_chars_in_order() { ::tsox_core::fntrace::enter("test_contains_chars_in_order"); 
    assert!(contains_chars_in_order("CamelCase", "cc"));
    assert!(contains_chars_in_order("hello world", "hw"));
    assert!(!contains_chars_in_order("hello world", "wh"));
}
