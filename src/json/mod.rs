//! JSON utilities, ported from `internal/json/`.
//!
//! In the Go version, this is a thin wrapper around `go-json-experiment`.
//! In Rust, we use `serde_json` directly, re-exporting the key types and
//! providing helper functions that mirror the Go API surface.

pub use serde_json::{
    from_slice, from_str, to_string, to_string_pretty, to_value, to_vec, to_vec_pretty,
    Map, Value, Deserializer, Serializer,
};

use std::io::Write;

/// Serialize a value to a JSON string with indentation.
pub fn marshal_indent<T: serde::Serialize>(value: &T, indent: &str) -> Result<String, serde_json::Error> {
    if indent.is_empty() {
        return marshal(value);
    }
    let buf = serde_json::to_vec_pretty(value)?;
    // serde_json already uses 2-space indentation; re-format with the given indent
    let s = String::from_utf8_lossy(&buf);
    Ok(reindent(&s, indent))
}

/// Serialize a value to a JSON string.
pub fn marshal<T: serde::Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

/// Deserialize a JSON string into a value.
pub fn unmarshal<'de, T: serde::Deserialize<'de>>(data: &'de str) -> Result<T, serde_json::Error> {
    serde_json::from_str(data)
}

/// Deserialize a JSON byte slice into a value.
pub fn unmarshal_slice<'de, T: serde::Deserialize<'de>>(data: &'de [u8]) -> Result<T, serde_json::Error> {
    serde_json::from_slice(data)
}

/// Serialize a value and write it to a writer.
pub fn marshal_write<W: Write, T: serde::Serialize>(writer: &mut W, value: &T) -> Result<(), serde_json::Error> {
    serde_json::to_writer(writer, value)
}

/// Serialize a value with indentation and write it to a writer.
pub fn marshal_indent_write<W: Write, T: serde::Serialize>(
    writer: &mut W,
    value: &T,
    indent: &str,
) -> Result<(), serde_json::Error> {
    if indent.is_empty() {
        return marshal_write(writer, value);
    }
    let s = marshal_indent(value, indent)?;
    writer.write_all(s.as_bytes()).map_err(|e| {
        serde_json::Error::io(e)
    })
}

/// Re-indent pretty-printed JSON, replacing 2-space indentation with the given indent string.
fn reindent(s: &str, indent: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for line in s.lines() {
        let trimmed = line.trim_start_matches(' ');
        let leading = line.len() - trimmed.len();
        let level = leading / 2;
        for _ in 0..level {
            result.push_str(indent);
        }
        result.push_str(trimmed);
        result.push('\n');
    }
    // Remove trailing newline
    if result.ends_with('\n') {
        result.pop();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[test]
    fn marshal_and_unmarshal() {
        let p = Point { x: 1, y: 2 };
        let json = marshal(&p).unwrap();
        assert_eq!(json, r#"{"x":1,"y":2}"#);
        let p2: Point = unmarshal(&json).unwrap();
        assert_eq!(p, p2);
    }

    #[test]
    fn marshal_indent_works() {
        let p = Point { x: 1, y: 2 };
        let json = marshal_indent(&p, "  ").unwrap();
        assert!(json.contains("\n"));
        assert!(json.contains("\"x\": 1"));
    }
}
