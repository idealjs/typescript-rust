use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use crate::lsp::lsproto_lsp_basic::MarkupKind;
use crate::lsp::lsproto_lsp_basic::StringOrMarkupContent;
use crate::lsp::lsproto_lsp_protocol::Diagnostic;
use crate::lsp::lsproto_lsp_protocol::IntegerOrString;
use crate::lsp::lsproto_util::compare_ranges;

pub struct StructFieldSpec {
    pub name: &'static str,
    pub required_id: i32,
    pub reject_null: bool,
}

pub struct StructSpec {
    pub by_name: HashMap<&'static str, StructFieldSpec>,
    pub required_names: Vec<&'static str>,
    pub required_mask: u64,
}

static STRUCT_SPEC_CACHE: LazyLock<Mutex<HashMap<TypeId, Arc<StructSpec>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn spec_for(key: TypeId, fields: &[StructFieldSpec]) -> Arc<StructSpec> { ::tsox_core::fntrace::enter("spec_for"); 
    if let Some(cached) = STRUCT_SPEC_CACHE.lock().unwrap().get(&key) {
        return cached.clone();
    }
    let mut by_name = HashMap::with_capacity(fields.len());
    let mut required_names = Vec::new();
    let mut required_mask = 0u64;
    for field in fields {
        let mut field_spec = StructFieldSpec {
            name: field.name,
            required_id: -1,
            reject_null: field.reject_null,
        };
        if field.required_id >= 0 {
            field_spec.required_id = required_names.len() as i32;
            required_mask |= 1 << field_spec.required_id;
            required_names.push(field.name);
        }
        by_name.insert(field.name, field_spec);
    }
    let spec = Arc::new(StructSpec {
        by_name,
        required_names,
        required_mask,
    });
    STRUCT_SPEC_CACHE
        .lock()
        .unwrap()
        .insert(key, spec.clone());
    spec
}

pub fn unmarshal_struct(data: &[u8], spec: &StructSpec) -> Result<serde_json::Value, String> { ::tsox_core::fntrace::enter("unmarshal_struct"); 
    let value: serde_json::Value = match serde_json::from_slice(data) {
        Ok(value) => value,
        Err(e) => return Err(e.to_string()),
    };
    let object = match value.as_object() {
        Some(object) => object,
        None => return Err(crate::mig::m5m::err_not_object(describe_kind(&value))),
    };

    let mut seen = 0u64;
    for (name, field_value) in object {
        let field_spec = match spec.by_name.get(name.as_str()) {
            Some(field_spec) => field_spec,
            None => continue,
        };
        if field_spec.required_id >= 0 {
            seen |= 1 << field_spec.required_id;
        }
        if field_spec.reject_null && field_value.is_null() {
            return Err(crate::mig::m5m::err_null(name));
        }
    }

    let missing = spec.required_mask & !seen;
    if missing != 0 {
        let missing_props: Vec<&str> = spec
            .required_names
            .iter()
            .enumerate()
            .filter(|(id, _)| missing & (1 << id) != 0)
            .map(|(_, name)| *name)
            .collect();
        return Err(crate::mig::m5m::err_missing(&missing_props));
    }
    Ok(value)
}

fn describe_kind(value: &serde_json::Value) -> &'static str { ::tsox_core::fntrace::enter("describe_kind"); 
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

pub fn marshal_union(
    value: &serde_json::Value,
    name: &str,
    nullable: bool,
) -> Result<serde_json::Value, String> { ::tsox_core::fntrace::enter("marshal_union"); 
    let object = match value.as_object() {
        Some(object) => object,
        None => return Err(crate::mig::m5m::err_not_object("value")),
    };
    let mut set: Option<&serde_json::Value> = None;
    let mut count = 0usize;
    for (_, field) in object {
        if !field.is_null() {
            count += 1;
            if set.is_none() {
                set = Some(field);
            }
        }
    }
    if nullable {
        crate::mig::m5m::assert_at_most_one(
            &format!("more than one element of {name} is set"),
            count,
        );
        if set.is_none() {
            return Ok(serde_json::Value::Null);
        }
    } else {
        crate::mig::m5m::assert_only_one(
            &format!("exactly one element of {name} should be set"),
            count,
        );
    }
    Ok(set.cloned().unwrap_or(serde_json::Value::Null))
}

pub fn count_non_nil(value: &serde_json::Value) -> usize { ::tsox_core::fntrace::enter("count_non_nil"); 
    match value.as_object() {
        Some(object) => object.values().filter(|field| !field.is_null()).count(),
        None => 0,
    }
}

pub fn integer_or_string_as_string(m: &IntegerOrString) -> String { ::tsox_core::fntrace::enter("integer_or_string_as_string"); 
    if let Some(s) = &m.string {
        return s.clone();
    }
    if let Some(i) = &m.integer {
        return i.to_string();
    }
    "-1".to_string()
}

pub fn diagnostic_exists_in_slice(elem: &Diagnostic, diags: &[Diagnostic]) -> bool { ::tsox_core::fntrace::enter("diagnostic_exists_in_slice"); 
    diags.iter().any(|diag| diagnostics_equal(elem, diag))
}

pub fn diagnostics_equal(diag1: &Diagnostic, diag2: &Diagnostic) -> bool { ::tsox_core::fntrace::enter("diagnostics_equal"); 
    diagnostic_codes_equal(&diag1.code, &diag2.code)
        && diagnostic_messages_equal_str(&diag1.message, &diag2.message)
        && compare_ranges(&diag1.range, &diag2.range) == std::cmp::Ordering::Equal
}

pub fn diagnostic_codes_equal(
    code1: &Option<serde_json::Value>,
    code2: &Option<serde_json::Value>,
) -> bool { ::tsox_core::fntrace::enter("diagnostic_codes_equal"); 
    match (code1, code2) {
        (Some(c1), Some(c2)) => match (c1.as_str(), c2.as_str()) {
            (Some(s1), Some(s2)) => s1 == s2,
            (None, None) => match (c1.as_i64(), c2.as_i64()) {
                (Some(i1), Some(i2)) => i1 == i2,
                _ => false,
            },
            _ => false,
        },
        _ => false,
    }
}

pub fn diagnostic_messages_equal_str(message1: &str, message2: &str) -> bool { ::tsox_core::fntrace::enter("diagnostic_messages_equal_str"); 
    message1 == message2
}

pub fn diagnostic_messages_equal(
    message1: &StringOrMarkupContent,
    message2: &StringOrMarkupContent,
) -> bool { ::tsox_core::fntrace::enter("diagnostic_messages_equal"); 
    match (&message1.string, &message2.string) {
        (Some(s1), Some(s2)) => s1 == s2,
        _ => match (&message1.markup_content, &message2.markup_content) {
            (Some(m1), Some(m2)) => m1.kind == m2.kind && m1.value == m2.value,
            _ => false,
        },
    }
}

pub fn compare_diagnostics<'a>(
    list1: &'a [Diagnostic],
    list2: &'a [Diagnostic],
) -> (Vec<&'a Diagnostic>, Vec<&'a Diagnostic>) { ::tsox_core::fntrace::enter("compare_diagnostics"); 
    let mut missing_from_list1 = Vec::new();
    let mut missing_from_list2 = Vec::new();
    for elem in list1 {
        if !diagnostic_exists_in_slice(elem, list2) {
            missing_from_list2.push(elem);
        }
    }
    for elem in list2 {
        if !diagnostic_exists_in_slice(elem, list1) {
            missing_from_list1.push(elem);
        }
    }
    (missing_from_list1, missing_from_list2)
}

pub fn diagnostic_as_string(elem: &Diagnostic) -> String { ::tsox_core::fntrace::enter("diagnostic_as_string"); 
    format!(
        "{} ({}:{}-{}:{}): {}",
        diagnostic_code_as_string(elem),
        elem.range.start.line,
        elem.range.start.character,
        elem.range.end.line,
        elem.range.end.character,
        elem.message
    )
}

pub fn diagnostic_code_as_string(elem: &Diagnostic) -> String { ::tsox_core::fntrace::enter("diagnostic_code_as_string"); 
    match &elem.code {
        Some(code) => format!("Code({})", code),
        None => "Code(-1)".to_string(),
    }
}

pub fn markup_kind_preferred(formats: &[MarkupKind]) -> MarkupKind { ::tsox_core::fntrace::enter("markup_kind_preferred"); 
    crate::mig::m5m::preferred_markup_kind(formats)
}
