pub use serde_json::{
    Deserializer, Map, Serializer, Value, from_slice, from_str, to_string, to_string_pretty,
    to_value, to_vec, to_vec_pretty,
};

use std::io::Write;

pub fn marshal_indent<T: serde::Serialize>(
    value: &T,
    indent: &str,
) -> Result<String, serde_json::Error> {
    if indent.is_empty() {
        return marshal(value);
    }
    let buf = serde_json::to_vec_pretty(value)?;

    let s = String::from_utf8_lossy(&buf);
    Ok(reindent(&s, indent))
}

pub fn marshal<T: serde::Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

pub fn unmarshal<'de, T: serde::Deserialize<'de>>(data: &'de str) -> Result<T, serde_json::Error> {
    serde_json::from_str(data)
}

pub fn unmarshal_slice<'de, T: serde::Deserialize<'de>>(
    data: &'de [u8],
) -> Result<T, serde_json::Error> {
    serde_json::from_slice(data)
}

pub fn marshal_write<W: Write, T: serde::Serialize>(
    writer: &mut W,
    value: &T,
) -> Result<(), serde_json::Error> {
    serde_json::to_writer(writer, value)
}

pub fn marshal_indent_write<W: Write, T: serde::Serialize>(
    writer: &mut W,
    value: &T,
    indent: &str,
) -> Result<(), serde_json::Error> {
    if indent.is_empty() {
        return marshal_write(writer, value);
    }
    let s = marshal_indent(value, indent)?;
    writer
        .write_all(s.as_bytes())
        .map_err(|e| serde_json::Error::io(e))
}

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

    if result.ends_with('\n') {
        result.pop();
    }
    result
}

#[cfg(test)]
mod tests;
