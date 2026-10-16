use crate::core::text::TextPos;
use crate::stringutil::is_line_break;
use crate::tspath::{
    EXTENSION_CJS, EXTENSION_CTS, EXTENSION_JS, EXTENSION_JSX, EXTENSION_JSON, EXTENSION_MJS,
    EXTENSION_MTS, EXTENSION_TS, EXTENSION_TSX,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScriptKind {
    #[default]
    Unknown = 0,
    Js = 1,
    Jsx = 2,
    Ts = 3,
    Tsx = 4,
    Json = 6,
}

pub fn filter_seq<'a, T: Clone>(
    slice: &'a [T],
    f: impl Fn(&'a T) -> bool + 'a,
) -> impl Iterator<Item = T> + 'a { crate::fntrace::enter("filter_seq"); 
    slice.iter().filter(move |v| f(v)).cloned()
}

pub fn filter_index<T: Clone>(slice: &[T], f: impl Fn(&T, usize, &[T]) -> bool) -> Vec<T> { crate::fntrace::enter("filter_index"); 
    for (i, value) in slice.iter().enumerate() {
        if !f(value, i, slice) {
            let mut result = slice[..i].to_vec();
            for (j, value) in slice.iter().enumerate().skip(i + 1) {
                if f(value, j, slice) {
                    result.push(value.clone());
                }
            }
            return result;
        }
    }
    slice.to_vec()
}

pub fn map_non_nil<T, U: Default + PartialEq>(slice: &[T], f: impl Fn(&T) -> U) -> Vec<U> { crate::fntrace::enter("map_non_nil"); 
    let mut result = Vec::new();
    for value in slice {
        let mapped = f(value);
        if mapped != U::default() {
            result.push(mapped);
        }
    }
    result
}

pub fn same_map<T: Clone + PartialEq>(slice: &[T], f: impl Fn(&T) -> T) -> Vec<T> { crate::fntrace::enter("same_map"); 
    for (i, value) in slice.iter().enumerate() {
        let mapped = f(value);
        if &mapped != value {
            let mut result = slice[..i].to_vec();
            result.push(mapped);
            for value in &slice[i + 1..] {
                result.push(f(value));
            }
            return result;
        }
    }
    slice.to_vec()
}

pub fn same_map_index<T: Clone + PartialEq>(slice: &[T], f: impl Fn(&T, usize) -> T) -> Vec<T> { crate::fntrace::enter("same_map_index"); 
    for (i, value) in slice.iter().enumerate() {
        let mapped = f(value, i);
        if &mapped != value {
            let mut result = slice[..i].to_vec();
            result.push(mapped);
            for (j, value) in slice.iter().enumerate().skip(i + 1) {
                result.push(f(value, j));
            }
            return result;
        }
    }
    slice.to_vec()
}

pub fn same<T>(s1: &[T], s2: &[T]) -> bool { crate::fntrace::enter("same"); 
    if s1.len() == s2.len() {
        return s1.is_empty() || s1.as_ptr() == s2.as_ptr();
    }
    false
}

pub fn or<T>(funcs: Vec<Box<dyn Fn(&T) -> bool>>) -> impl Fn(&T) -> bool { crate::fntrace::enter("or"); 
    move |input: &T| funcs.iter().any(|f| f(input))
}

pub fn first_or_nil<T: Clone>(slice: &[T]) -> Option<T> { crate::fntrace::enter("first_or_nil"); 
    slice.first().cloned()
}

pub fn last_or_nil<T: Clone>(slice: &[T]) -> Option<T> { crate::fntrace::enter("last_or_nil"); 
    slice.last().cloned()
}

pub fn element_or_nil<T: Clone>(slice: &[T], index: usize) -> Option<T> { crate::fntrace::enter("element_or_nil"); 
    if index < slice.len() {
        Some(slice[index].clone())
    } else {
        None
    }
}

pub fn first_or_nil_seq<T, I: Iterator<Item = T>>(seq: I) -> Option<T> { crate::fntrace::enter("first_or_nil_seq"); 
    seq.into_iter().next()
}

pub fn first_non_nil<T, U: Default + PartialEq>(slice: &[T], f: impl Fn(&T) -> U) -> U { crate::fntrace::enter("first_non_nil"); 
    for value in slice {
        let mapped = f(value);
        if mapped != U::default() {
            return mapped;
        }
    }
    U::default()
}

pub fn concatenate_seq<T, I: IntoIterator<Item = I2>, I2: IntoIterator<Item = T>>(
    seqs: I,
) -> impl Iterator<Item = T> { crate::fntrace::enter("concatenate_seq"); 
    seqs.into_iter().flatten()
}

pub fn enumerate<T, I: Iterator<Item = T>>(seq: I) -> std::iter::Enumerate<I> { crate::fntrace::enter("enumerate"); 
    seq.enumerate()
}

pub fn if_else<T>(b: bool, when_true: T, when_false: T) -> T { crate::fntrace::enter("if_else"); 
    if b {
        when_true
    } else {
        when_false
    }
}

pub fn or_else<T: Default + PartialEq>(value: T, default_value: T) -> T { crate::fntrace::enter("or_else"); 
    if value != T::default() {
        value
    } else {
        default_value
    }
}

pub fn coalesce<T>(a: Option<T>, b: Option<T>) -> Option<T> { crate::fntrace::enter("coalesce"); 
    a.or(b)
}

pub type EcmaLineStarts = Vec<TextPos>;

pub fn compute_ecma_line_starts(text: &str) -> EcmaLineStarts { crate::fntrace::enter("compute_ecma_line_starts"); 
    compute_ecma_line_starts_seq(text).collect()
}

pub fn compute_ecma_line_starts_seq(text: &str) -> impl Iterator<Item = TextPos> + '_ { crate::fntrace::enter("compute_ecma_line_starts_seq"); 
    let mut pos: TextPos = 0;
    let mut line_start: TextPos = 0;
    let text_len = text.len() as TextPos;
    let mut pending = Vec::new();
    let bytes = text.as_bytes();
    while pos < text_len {
        let b = bytes[pos as usize];
        if b < 0x80 {
            pos += 1;
            if b == b'\r' {
                if pos < text_len && bytes[pos as usize] == b'\n' {
                    pos += 1;
                }
                pending.push(line_start);
                line_start = pos;
            } else if b == b'\n' {
                pending.push(line_start);
                line_start = pos;
            }
        } else {
            let ch = text[pos as usize..].chars().next().unwrap();
            pos += ch.len_utf8() as TextPos;
            if is_line_break(ch) {
                pending.push(line_start);
                line_start = pos;
            }
        }
    }
    pending.push(line_start);
    pending.into_iter()
}

pub fn position_to_line_and_byte_offset(position: usize, line_starts: &[TextPos]) -> (usize, usize) { crate::fntrace::enter("position_to_line_and_byte_offset"); 
    let line = match line_starts
        .binary_search_by(|probe| probe.cmp(&(position as TextPos)))
    {
        Ok(idx) => idx,
        Err(idx) => idx.saturating_sub(1),
    };
    (line, position - line_starts[line] as usize)
}

pub fn flatten<T: Clone>(array: &[Vec<T>]) -> Vec<T> { crate::fntrace::enter("flatten"); 
    array.concat()
}

pub fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T { crate::fntrace::enter("must"); 
    result.unwrap()
}

pub fn first_result<T1>(t1: T1, _rest: &[&dyn std::any::Any]) -> T1 { crate::fntrace::enter("first_result"); 
    t1
}

pub fn stringify_json<T: serde::Serialize>(
    input: &T,
    prefix: &str,
    indent: &str,
) -> Result<String, serde_json::Error> { crate::fntrace::enter("stringify_json"); 
    serde_json::to_string_pretty(input).map(|s| {
        let _ = (prefix, indent);
        s
    })
}

pub fn get_script_kind_from_file_name(file_name: &str) -> ScriptKind { crate::fntrace::enter("get_script_kind_from_file_name"); 
    if let Some(dot_pos) = file_name.rfind('.') {
        match file_name[dot_pos..].to_lowercase().as_str() {
            EXTENSION_JS | EXTENSION_CJS | EXTENSION_MJS => ScriptKind::Js,
            EXTENSION_JSX => ScriptKind::Jsx,
            EXTENSION_TS | EXTENSION_CTS | EXTENSION_MTS => ScriptKind::Ts,
            EXTENSION_TSX => ScriptKind::Tsx,
            EXTENSION_JSON => ScriptKind::Json,
            _ => ScriptKind::Unknown,
        }
    } else {
        ScriptKind::Unknown
    }
}

pub fn get_default_extension_for_script_kind(script_kind: ScriptKind) -> &'static str { crate::fntrace::enter("get_default_extension_for_script_kind"); 
    match script_kind {
        ScriptKind::Js => EXTENSION_JS,
        ScriptKind::Jsx => EXTENSION_JSX,
        ScriptKind::Tsx => EXTENSION_TSX,
        ScriptKind::Json => EXTENSION_JSON,
        _ => EXTENSION_TS,
    }
}

pub fn ensure_script_kind_from_file_name(file_name: &str) -> ScriptKind { crate::fntrace::enter("ensure_script_kind_from_file_name"); 
    let kind = get_script_kind_from_file_name(file_name);
    if kind != ScriptKind::Unknown {
        return kind;
    }
    ScriptKind::Ts
}
