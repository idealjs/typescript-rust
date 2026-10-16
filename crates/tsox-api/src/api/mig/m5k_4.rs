#![allow(unused_imports, dead_code)]

use std::sync::Arc;

use serde_json::Value as JsonValue;

use super::m5k_3::{SymbolId, TypeId};
use tsox_checker::checker::{
    is_tuple_type, LiteralValue, ObjectFlags, Type, TypeFlags, TYPE_FLAGS_FRESHABLE,
    TYPE_FLAGS_INTRINSIC, TYPE_FLAGS_LITERAL, TYPE_FLAGS_UNION_OR_INTERSECTION,
};
use tsox_core::core::text::TextRange;
use tsox_core::diagnostics::Category;
use tsox_core::locale::Locale;
use tsox_frontend::ast::mig::m3d_2::new_diagnostic_from_text;
use tsox_frontend::ast::{positionmap, Diagnostic, SourceFile};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticPositionResponse {
    pub line: i32,
    pub character: i32,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticSourceLineResponse {
    pub line: i32,
    pub text: String,
}

pub fn diagnostic_source_lines(
    file: &SourceFile,
    first_line: i32,
    last_line: i32,
) -> Vec<DiagnosticSourceLineResponse> { ::tsox_core::fntrace::enter("diagnostic_source_lines"); 
    let line_map = &file.line_map.line_starts;
    if line_map.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<i32> =
        Vec::with_capacity(std::cmp::min(last_line - first_line + 1, 4) as usize);
    if last_line - first_line >= 4 {
        lines.push(first_line);
        lines.push(first_line + 1);
        lines.push(last_line - 1);
        lines.push(last_line);
    } else {
        for line in first_line..=last_line {
            lines.push(line);
        }
    }
    let text = &file.text;
    let mut result = Vec::with_capacity(lines.len());
    for &line in &lines {
        let start = line_map.get(line.max(0) as usize).copied().unwrap_or(0) as usize;
        let end = if (line as usize) + 1 < line_map.len() {
            line_map[(line + 1) as usize] as usize
        } else {
            text.len()
        };
        result.push(DiagnosticSourceLineResponse {
            line,
            text: text[start..end].to_string(),
        });
    }
    result
}

pub fn new_diagnostic_response(d: &Diagnostic) -> DiagnosticResponse { ::tsox_core::fntrace::enter("new_diagnostic_response"); 
    new_diagnostic_response_from_wrapped(d)
}

pub fn new_diagnostic_response_from_wrapped(d: &Diagnostic) -> DiagnosticResponse { ::tsox_core::fntrace::enter("new_diagnostic_response_from_wrapped"); 
    let file = d.file();
    let mut pos = d.pos();
    let mut end = d.end();
    if let Some(file) = &file {
        let text_len = file.text.len() as i32;
        pos = std::cmp::max(0, std::cmp::min(pos, text_len));
        end = std::cmp::max(pos, std::cmp::min(end, text_len));
    }
    let source = d.source();
    let mut resp = DiagnosticResponse {
        pos,
        end,
        code: d.code(),
        category: d.category().name().to_string(),
        source: if source.is_empty() {
            None
        } else {
            Some(source.to_string())
        },
        text: d.localize(Locale::default_locale()),
        reports_unnecessary: d.reports_unnecessary(),
        reports_deprecated: d.reports_deprecated(),
        file_name: None,
        start_position: None,
        end_position: None,
        source_lines: Vec::new(),
        message_chain: Vec::new(),
        related_information: Vec::new(),
    };
    if let Some(file) = &file {
        resp.file_name = Some(file.file_name.clone());
        let pos = pos.max(0) as usize;
        let end = end.max(0) as usize;
        let position_map = positionmap::compute_position_map(&file.text);
        resp.pos = position_map.utf8_to_utf16(pos) as i32;
        resp.end = position_map.utf8_to_utf16(end) as i32;
        let start_line = file.line_map.line_at(pos) as i32;
        let start_character = file.line_map.utf16_column_at(&file.text, pos) as i32;
        let end_line = file.line_map.line_at(end) as i32;
        let end_character = file.line_map.utf16_column_at(&file.text, end) as i32;
        resp.start_position = Some(DiagnosticPositionResponse {
            line: start_line,
            character: start_character,
        });
        resp.end_position = Some(DiagnosticPositionResponse {
            line: end_line,
            character: end_character,
        });
        resp.source_lines = diagnostic_source_lines(file, start_line, end_line);
    }
    let chain = d.message_chain();
    if !chain.is_empty() {
        resp.message_chain = chain.iter().map(new_diagnostic_response).collect();
    }
    let related = d.related_information();
    if !related.is_empty() {
        resp.related_information = related.iter().map(new_diagnostic_response).collect();
    }
    resp
}

pub fn new_diagnostic_responses(diags: &[Diagnostic]) -> Option<Vec<DiagnosticResponse>> { ::tsox_core::fntrace::enter("new_diagnostic_responses"); 
    if diags.is_empty() {
        return None;
    }
    Some(diags.iter().map(new_diagnostic_response).collect())
}

pub use crate::api::mig::m5l::DiagnosticResponse;

fn category_from_name(name: &str) -> Category { ::tsox_core::fntrace::enter("category_from_name"); 
    match name {
        "error" => Category::Error,
        "suggestion" => Category::Suggestion,
        "message" => Category::Message,
        _ => Category::Warning,
    }
}

impl DiagnosticResponse {
    pub fn to_diagnostic(&self) -> Diagnostic { ::tsox_core::fntrace::enter("to_diagnostic"); 
        new_diagnostic_from_text(
            None,
            TextRange::new(self.pos.max(0) as usize, self.end.max(0) as usize),
            self.code,
            category_from_name(&self.category),
            self.text.clone(),
            self.message_chain
                .iter()
                .map(|d| crate::api::mig::m5l::DiagnosticResponse::to_diagnostic(d))
                .collect(),
            self.related_information
                .iter()
                .map(|d| crate::api::mig::m5l::DiagnosticResponse::to_diagnostic(d))
                .collect(),
            self.reports_unnecessary,
            self.reports_deprecated,
        )
    }
}

fn symbol_handle(symbol: &tsox_frontend::ast::Symbol) -> SymbolId { ::tsox_core::fntrace::enter("symbol_handle"); 
    symbol.id() as u32
}

fn type_handle(t: &Type) -> TypeId { ::tsox_core::fntrace::enter("type_handle"); 
    t.id
}

fn type_handles(types: &[Arc<Type>]) -> Option<Vec<TypeId>> { ::tsox_core::fntrace::enter("type_handles"); 
    if types.is_empty() {
        None
    } else {
        Some(types.iter().map(|t| t.id).collect())
    }
}

fn literal_value_to_json(value: &LiteralValue) -> JsonValue { ::tsox_core::fntrace::enter("literal_value_to_json"); 
    match value {
        LiteralValue::String(s) => JsonValue::String(s.clone()),
        LiteralValue::Number(n) => JsonValue::from(n.0),
        LiteralValue::BigInt(b) => JsonValue::String(b.to_string()),
        LiteralValue::Boolean(b) => JsonValue::Bool(*b),
        LiteralValue::None => JsonValue::Null,
    }
}

fn object_flags_data(t: &Type) -> Option<&tsox_checker::checker::ObjectTypeData> { ::tsox_core::fntrace::enter("object_flags_data"); 
    match &t.data {
        tsox_checker::checker::TypeData::Object(data) => Some(data),
        tsox_checker::checker::TypeData::Interface(data) => Some(&data.object),
        tsox_checker::checker::TypeData::Tuple(data) => Some(&data.interface_data.object),
        tsox_checker::checker::TypeData::Mapped(data) => Some(&data.object),
        tsox_checker::checker::TypeData::ReverseMapped(data) => Some(&data.object),
        tsox_checker::checker::TypeData::EvolvingArray(data) => Some(&data.object),
        tsox_checker::checker::TypeData::InstantiationExpression(data) => Some(&data.object),
        _ => None,
    }
}

pub fn new_type_response(t: &Type, id: u32) -> TypeResponse { ::tsox_core::fntrace::enter("new_type_response"); 
    let mut resp = TypeResponse {
        id,
        flags: t.flags.bits(),
        symbol: None,
        alias_type_arguments: None,
        alias_symbol: None,
        value: JsonValue::Null,
        fresh_type: None,
        regular_type: None,
        object_flags: 0,
        is_tuple_type: false,
        element_flags: Vec::new(),
        fixed_length: None,
        tuple_readonly: None,
        target: None,
        type_parameters: None,
        outer_type_parameters: None,
        local_type_parameters: None,
        object_type: None,
        index_type: None,
        check_type: None,
        extends_type: None,
        base_type: None,
        subst_constraint: None,
        texts: Vec::new(),
        is_this_type: false,
        intrinsic_name: None,
        labeled_element_declarations: None,
    };
    if let Some(symbol) = &t.symbol {
        resp.symbol = Some(symbol_handle(symbol));
    }
    if let Some(alias) = &t.alias {
        resp.alias_type_arguments = type_handles(&alias.type_arguments);
        if let Some(alias_symbol) = &alias.symbol {
            resp.alias_symbol = Some(symbol_handle(alias_symbol));
        }
    }
    let flags = t.flags;
    if flags.intersects(TYPE_FLAGS_FRESHABLE) {
        if let tsox_checker::checker::TypeData::Literal(lit) = &t.data {
            if flags.intersects(TYPE_FLAGS_LITERAL) {
                resp.value = literal_value_to_json(&lit.value);
            }
            if let Some(fresh) = lit.fresh_type.get() {
                resp.fresh_type = Some(type_handle(fresh));
            }
            if let Some(regular) = lit.regular_type.get() {
                resp.regular_type = Some(type_handle(regular));
            }
        }
    } else if flags.intersects(TypeFlags::Object) {
        resp.object_flags = t.object_flags.bits();
        resp.is_tuple_type = is_tuple_type(t);
        if t.object_flags.intersects(ObjectFlags::Reference) {
            if let tsox_checker::checker::TypeData::Tuple(tuple) = &t.data {
                resp.element_flags = tuple
                    .element_infos
                    .iter()
                    .map(|info| info.flags.bits())
                    .collect();
                resp.fixed_length = Some(tuple.fixed_length as u32);
                resp.tuple_readonly = Some(tuple.readonly);
            }
            if let Some(object) = object_flags_data(t) {
                if let Some(target) = &object.target {
                    resp.target = Some(type_handle(target));
                }
            }
        }
        if t.object_flags
            .intersects(ObjectFlags::Class | ObjectFlags::Interface)
        {
            if let tsox_checker::checker::TypeData::Interface(interface) = &t.data {
                let all = &interface.all_type_parameters;
                if !all.is_empty() {
                    let end = all.len() - 1;
                    let outer = interface.outer_type_parameter_count.min(end);
                    resp.type_parameters = type_handles(&all[..end]);
                    resp.outer_type_parameters = type_handles(&all[..outer]);
                    resp.local_type_parameters = type_handles(&all[outer..end]);
                }
            }
        }
    } else if flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION) {
    } else if flags.intersects(TypeFlags::Index) {
        if let tsox_checker::checker::TypeData::Index(data) = &t.data {
            if let Some(target) = &data.target {
                resp.target = Some(type_handle(target));
            }
        }
    } else if flags.intersects(TypeFlags::IndexedAccess) {
        if let tsox_checker::checker::TypeData::IndexedAccess(data) = &t.data {
            if let Some(object_type) = &data.object_type {
                resp.object_type = Some(type_handle(object_type));
            }
            if let Some(index_type) = &data.index_type {
                resp.index_type = Some(type_handle(index_type));
            }
        }
    } else if flags.intersects(TypeFlags::Conditional) {
        if let tsox_checker::checker::TypeData::Conditional(data) = &t.data {
            if let Some(check_type) = &data.check_type {
                resp.check_type = Some(type_handle(check_type));
            }
            if let Some(extends_type) = &data.extends_type {
                resp.extends_type = Some(type_handle(extends_type));
            }
        }
    } else if flags.intersects(TypeFlags::Substitution) {
        if let tsox_checker::checker::TypeData::Substitution(data) = &t.data {
            if let Some(base_type) = &data.base_type {
                resp.base_type = Some(type_handle(base_type));
            }
            if let Some(subst_constraint) = &data.constraint {
                resp.subst_constraint = Some(type_handle(subst_constraint));
            }
        }
    } else if flags.intersects(TypeFlags::TemplateLiteral) {
        if let tsox_checker::checker::TypeData::TemplateLiteral(tl) = &t.data {
            resp.texts = tl.texts.clone();
        }
    } else if flags.intersects(TypeFlags::StringMapping) {
        if let tsox_checker::checker::TypeData::StringMapping(data) = &t.data {
            if let Some(target) = &data.target {
                resp.target = Some(type_handle(target));
            }
        }
    } else if flags.intersects(TypeFlags::TypeParameter) {
        if let tsox_checker::checker::TypeData::TypeParameter(data) = &t.data {
            resp.is_this_type = data.is_this_type;
        }
    } else if flags.intersects(TYPE_FLAGS_INTRINSIC) {
        if let tsox_checker::checker::TypeData::Intrinsic(data) = &t.data {
            resp.intrinsic_name = Some(data.intrinsic_name.clone());
        }
    }
    resp
}

#[derive(serde::Serialize)]
pub struct TypeResponse {
    pub id: u32,
    pub flags: u32,
    pub symbol: Option<SymbolId>,
    pub alias_type_arguments: Option<Vec<TypeId>>,
    pub alias_symbol: Option<SymbolId>,
    pub value: JsonValue,
    pub fresh_type: Option<TypeId>,
    pub regular_type: Option<TypeId>,
    pub object_flags: u32,
    pub is_tuple_type: bool,
    pub element_flags: Vec<u32>,
    pub fixed_length: Option<u32>,
    pub tuple_readonly: Option<bool>,
    pub target: Option<TypeId>,
    pub type_parameters: Option<Vec<TypeId>>,
    pub outer_type_parameters: Option<Vec<TypeId>>,
    pub local_type_parameters: Option<Vec<TypeId>>,
    pub object_type: Option<TypeId>,
    pub index_type: Option<TypeId>,
    pub check_type: Option<TypeId>,
    pub extends_type: Option<TypeId>,
    pub base_type: Option<TypeId>,
    pub subst_constraint: Option<TypeId>,
    pub texts: Vec<String>,
    pub is_this_type: bool,
    pub intrinsic_name: Option<String>,
    pub labeled_element_declarations: Option<Vec<super::m5l::NodeHandle>>,
}
