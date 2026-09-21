#![allow(unused_imports)]

use crate::checker::string_mapping::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StringMappingKind {
    Uppercase,
    Lowercase,
    Capitalize,
    Uncapitalize,
}

pub fn string_mapping_kind(name: &str) -> Option<StringMappingKind> {
    match name {
        "Uppercase" => Some(StringMappingKind::Uppercase),
        "Lowercase" => Some(StringMappingKind::Lowercase),
        "Capitalize" => Some(StringMappingKind::Capitalize),
        "Uncapitalize" => Some(StringMappingKind::Uncapitalize),
        _ => None,
    }
}

fn map_ascii_char(kind: StringMappingKind, c: char) -> char {
    match kind {
        StringMappingKind::Uppercase | StringMappingKind::Capitalize => {
            c.to_ascii_uppercase()
        }
        StringMappingKind::Lowercase | StringMappingKind::Uncapitalize => {
            c.to_ascii_lowercase()
        }
    }
}

fn map_str(kind: StringMappingKind, s: &str) -> String {
    let mapped: String = s
        .chars()
        .map(|c| {
            if c.is_ascii() {
                map_ascii_char(kind, c).to_string()
            } else {
                match kind {
                    StringMappingKind::Uppercase | StringMappingKind::Capitalize => {
                        c.to_uppercase().collect()
                    }
                    _ => c.to_lowercase().collect(),
                }
            }
        })
        .collect();
    mapped
}

pub fn apply_string_mapping(kind: StringMappingKind, s: &str) -> String {
    match kind {
        StringMappingKind::Uppercase | StringMappingKind::Lowercase => map_str(kind, s),
        StringMappingKind::Capitalize | StringMappingKind::Uncapitalize => {
            if s.is_empty() {
                return s.to_string();
            }
            let mut chars = s.chars();
            let first = chars.next().expect("non-empty checked");
            let rest: String = chars.collect();
            let mapped_first = if first.is_ascii() {
                map_ascii_char(kind, first).to_string()
            } else {
                match kind {
                    StringMappingKind::Capitalize => first.to_uppercase().collect(),
                    _ => first.to_lowercase().collect(),
                }
            };
            format!("{mapped_first}{rest}")
        }
    }
}

impl Checker {
    pub fn intrinsic_marker_type(&self) -> Arc<Type> {
        self.intrinsic_marker_type
            .get_or_init(|| {
                Arc::new(Type {
                    flags: TypeFlags::Any,
                    object_flags: ObjectFlags::None,
                    id: crate::checker::types::next_type_id(),
                    symbol: None,
                    alias: None,
                    data: TypeData::Intrinsic(crate::checker::types::IntrinsicTypeData {
                        intrinsic_name: "intrinsic".to_string(),
                    }),
                })
            })
            .clone()
    }

    fn is_intrinsic_marker(&self, t: &Arc<Type>) -> bool {
        match self.intrinsic_marker_type.get() {
            Some(m) => Arc::ptr_eq(m, t),
            None => false,
        }
    }

    pub fn get_string_mapping_type(
        &mut self,
        kind: StringMappingKind,
        symbol: Option<Arc<tsox_frontend::ast::Symbol>>,
        t: &Arc<Type>,
    ) -> Arc<Type> {
        if t.is_union() {
            let parts: Vec<Arc<Type>> = t
                .types()
                .map(|ts| ts.to_vec())
                .unwrap_or_default()
                .iter()
                .map(|c| self.get_string_mapping_type(kind, symbol.clone(), c))
                .collect();
            return self.get_union_type(parts);
        }
        if t.flags.contains(TypeFlags::StringLiteral) {
            let text = t
                .literal_value()
                .and_then(|v| match v {
                    crate::checker::types::LiteralValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            return self.get_string_literal_type(&apply_string_mapping(kind, &text));
        }
        if t.flags.contains(TypeFlags::Never) {
            return Arc::clone(t);
        }
        if let TypeData::TemplateLiteral(tl) = &t.data {
            let (texts, types) = self.apply_template_string_mapping(kind, &tl.texts, &tl.types);
            return self.create_template_literal_type(texts, types);
        }
        if let TypeData::StringMapping(_) = &t.data {
            if symbol.as_ref().zip(t.symbol.as_ref()).is_some_and(|(a, b)| {
                a.name == b.name && string_mapping_kind(&b.name).is_some()
            }) || t.symbol.is_none() && symbol.is_none() {
                return Arc::clone(t);
            }
            return Arc::clone(t);
        }
        if t.flags.intersects(TypeFlags::Any | TypeFlags::String | TypeFlags::StringMapping) {
            let key = self.string_mapping_cache_key(kind, t);
            if let Some(cached) = self.string_mapping_types.get(&key) {
                return Arc::clone(cached);
            }
            let mapped = Arc::new(Type {
                flags: TypeFlags::StringMapping,
                object_flags: ObjectFlags::None,
                id: crate::checker::types::next_type_id(),
                symbol: symbol.clone(),
                alias: None,
                data: TypeData::StringMapping(crate::checker::types::StringMappingTypeData {
                    constrained: Default::default(),
                    target: Some(Arc::clone(t)),
                }),
            });
            self.string_mapping_types.insert(key, Arc::clone(&mapped));
            return mapped;
        }
        Arc::clone(t)
    }

    fn string_mapping_cache_key(&self, kind: StringMappingKind, t: &Arc<Type>) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        (kind as u8).hash(&mut h);
        t.id.hash(&mut h);
        h.finish()
    }

    fn apply_template_string_mapping(
        &mut self,
        kind: StringMappingKind,
        texts: &[String],
        types: &[Arc<Type>],
    ) -> (Vec<String>, Vec<Arc<Type>>) {
        match kind {
            StringMappingKind::Uppercase | StringMappingKind::Lowercase => {
                let new_texts = texts
                    .iter()
                    .map(|t| apply_string_mapping(kind, t))
                    .collect();
                let new_types = types
                    .iter()
                    .map(|t| {
                        self.get_string_mapping_type(kind, None, t)
                    })
                    .collect();
                (new_texts, new_types)
            }
            StringMappingKind::Capitalize | StringMappingKind::Uncapitalize => {
                if texts.first().is_some_and(|t| !t.is_empty()) {
                    let mut new_texts = texts.to_vec();
                    new_texts[0] = apply_string_mapping(kind, &new_texts[0]);
                    return (new_texts, types.to_vec());
                }
                let mut new_types = types.to_vec();
                if let Some(first) = new_types.first().cloned() {
                    new_types[0] = self.get_string_mapping_type(kind, None, &first);
                }
                (texts.to_vec(), new_types)
            }
        }
    }

    pub fn create_template_literal_type(
        &mut self,
        texts: Vec<String>,
        types: Vec<Arc<Type>>,
    ) -> Arc<Type> {
        let all_literal = types.iter().all(|t| {
            t.flags.intersects(
                TYPE_FLAGS_LITERAL | TypeFlags::Null | TypeFlags::Undefined,
            )
        });
        if all_literal && types.len() + 1 == texts.len() {
            let mut sb = texts.first().cloned().unwrap_or_default();
            for (t, tail) in types.iter().zip(texts.iter().skip(1)) {
                sb.push_str(&self.template_string_for_type(t));
                sb.push_str(tail);
            }
            return self.get_string_literal_type(&sb);
        }
        Arc::new(Type::new(
            TypeFlags::TemplateLiteral,
            TypeData::TemplateLiteral(crate::checker::types::TemplateLiteralTypeData {
                constrained: Default::default(),
                texts,
                types,
            }),
        ))
    }

    pub(crate) fn intrinsic_alias_instantiation(
        &mut self,
        symbol: &Arc<tsox_frontend::ast::Symbol>,
        arg_types: &[Arc<Type>],
    ) -> Option<Arc<Type>> {
        let kind = string_mapping_kind(&symbol.name)?;
        if arg_types.len() != 1 {
            return None;
        }
        let declared = self
            .type_alias_links
            .get(symbol)
            .and_then(|l| l.declared_type.clone())?;
        if !self.is_intrinsic_marker(&declared) {
            return None;
        }
        Some(self.get_string_mapping_type(kind, Some(Arc::clone(symbol)), &arg_types[0]))
    }
}
