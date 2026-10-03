use super::*;

fn matches(pattern: &str, input: &str) -> bool { crate::fntrace::enter("matches"); 
    Glob::parse(pattern)
        .unwrap_or_else(|e| panic!("Failed to parse pattern '{}': {}", pattern, e))
        .is_match(input)
}

#[test]
fn literal_match() { crate::fntrace::enter("literal_match"); 
    assert!(matches("foo", "foo"));
    assert!(!matches("foo", "bar"));
    assert!(!matches("foo", "foobar"));
}

#[test]
fn star_match() { crate::fntrace::enter("star_match"); 
    assert!(matches("*.ts", "foo.ts"));
    assert!(matches("*.ts", "bar.ts"));
    assert!(!matches("*.ts", "foo.js"));
    assert!(matches("a*", "abc"));
}

#[test]
fn question_match() { crate::fntrace::enter("question_match"); 
    assert!(matches("?.ts", "a.ts"));
    assert!(!matches("?.ts", "ab.ts"));
}

#[test]
fn starstar_match() { crate::fntrace::enter("starstar_match"); 
    assert!(matches("**/*.ts", "foo.ts"));
    assert!(matches("**/*.ts", "a/b/foo.ts"));
    assert!(matches("**/*.ts", "a/b/c/foo.ts"));
}

#[test]
fn group_match() { crate::fntrace::enter("group_match"); 
    assert!(matches("*.{ts,js}", "foo.ts"));
    assert!(matches("*.{ts,js}", "foo.js"));
    assert!(!matches("*.{ts,js}", "foo.json"));
}

#[test]
fn char_range_match() { crate::fntrace::enter("char_range_match"); 
    assert!(matches("example.[0-9]", "example.0"));
    assert!(matches("example.[0-9]", "example.9"));
    assert!(!matches("example.[0-9]", "example.a"));
}

#[test]
fn negated_range_match() { crate::fntrace::enter("negated_range_match"); 
    assert!(matches("example.[!0-9]", "example.a"));
    assert!(!matches("example.[!0-9]", "example.0"));
}

#[test]
fn slash_match() { crate::fntrace::enter("slash_match"); 
    assert!(matches("a/b", "a/b"));
    assert!(matches("a/b", "a//b"));
    assert!(!matches("a//b", "a/b"));
    assert!(!matches("a/b", "a/c"));
}
