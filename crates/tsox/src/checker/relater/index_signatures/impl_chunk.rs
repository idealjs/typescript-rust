#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn index_signatures_related_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        source_is_primitive: bool,
        relation: RelationKind,
    ) -> Ternary {
        if relation == RelationKind::Identity {
            return self.index_signatures_identical_to(source, target);
        }
        let target_indexes = self.get_index_infos_of_type(target);
        let target_has_string_index = target_indexes.iter().any(|info| {
            info.key_type
                .as_ref()
                .map(|k| k.flags.contains(TypeFlags::String))
                .unwrap_or(false)
        });
        let mut result = Ternary::True;
        for target_info in &target_indexes {
            let target_value_any = target_info
                .value_type
                .as_ref()
                .map(|v| v.flags.contains(TypeFlags::Any))
                .unwrap_or(false);
            let target_key_is_string = target_info
                .key_type
                .as_ref()
                .map(|k| k.flags.contains(TypeFlags::String))
                .unwrap_or(false);
            let related = if relation != RelationKind::StrictSubtype
                && !source_is_primitive
                && target_has_string_index
                && target_key_is_string
                && target_value_any
            {
                Ternary::True
            } else if self.is_generic_mapped_type(source) && target_key_is_string {
                let template = self.get_template_type_from_mapped_type(source);
                match template {
                    Some(template) => {
                        let target_value = target_info
                            .value_type
                            .clone()
                            .unwrap_or_else(|| self.any_type());
                        self.compare_types(template, target_value, relation, false)
                    }
                    None => Ternary::False,
                }
            } else {
                self.type_related_to_index_info(source, target_info, relation)
            };
            if related.is_false() {
                return Ternary::False;
            }
            result = result.and(related);
        }
        result
    }

    pub fn type_related_to_index_info(
        &mut self,
        source: &Arc<Type>,
        target_info: &IndexInfo,
        relation: RelationKind,
    ) -> Ternary {
        let target_key = match &target_info.key_type {
            Some(k) => k,
            None => return Ternary::True,
        };
        let source_info = self.get_applicable_index_info(source, target_key);
        if let Some(source_info) = source_info {
            return self.index_info_related_to(&source_info, target_info, relation);
        }

        let is_fresh_literal = source.object_flags.contains(ObjectFlags::FreshLiteral);
        if relation != RelationKind::StrictSubtype || is_fresh_literal {
            if self.is_object_type_with_inferable_index(source) {
                return self.members_related_to_index_info(source, target_info, relation);
            }
        }
        Ternary::False
    }

    pub fn index_info_related_to(
        &mut self,
        source_info: &IndexInfo,
        target_info: &IndexInfo,
        relation: RelationKind,
    ) -> Ternary {
        let source_value = source_info
            .value_type
            .clone()
            .unwrap_or_else(|| self.any_type());
        let target_value = target_info
            .value_type
            .clone()
            .unwrap_or_else(|| self.any_type());
        self.compare_types(source_value, target_value, relation, false)
    }

    pub fn index_signatures_identical_to(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Ternary {
        let source_infos = self.get_index_infos_of_type(source);
        let target_infos = self.get_index_infos_of_type(target);
        if source_infos.len() != target_infos.len() {
            return Ternary::False;
        }
        for target_info in &target_infos {
            let target_key = match &target_info.key_type {
                Some(k) => Arc::clone(k),
                None => continue,
            };
            let source_info = self.get_index_info_of_type(source, &target_key);
            let related = match source_info {
                Some(si) => {
                    let sv = si.value_type.clone().unwrap_or_else(|| self.any_type());
                    let tv = target_info
                        .value_type
                        .clone()
                        .unwrap_or_else(|| self.any_type());
                    let type_related = self.compare_types(sv, tv, RelationKind::Identity, false);
                    let readonly_match = si.is_readonly == target_info.is_readonly;
                    if type_related.is_true() && readonly_match {
                        Ternary::True
                    } else {
                        Ternary::False
                    }
                }
                None => Ternary::False,
            };
            if related.is_false() {
                return Ternary::False;
            }
        }
        Ternary::True
    }

    pub fn get_index_infos_of_type(&self, t: &Arc<Type>) -> Vec<Arc<IndexInfo>> {
        t.as_structured()
            .map(|s| s.index_infos.clone())
            .unwrap_or_default()
    }

    pub fn get_index_info_of_type(
        &self,
        t: &Arc<Type>,
        key_type: &Arc<Type>,
    ) -> Option<Arc<IndexInfo>> {
        for info in self.get_index_infos_of_type(t) {
            if let Some(info_key) = &info.key_type {
                if Arc::ptr_eq(info_key, key_type) || info_key.flags == key_type.flags {
                    return Some(info);
                }
            }
        }

        if key_type.flags.contains(TypeFlags::Number) {
            if let TypeData::Tuple(tuple) = &t.data {
                let elements: Vec<Arc<Type>> = tuple
                    .element_infos
                    .iter()
                    .filter_map(|e| e.type_.clone())
                    .collect();
                if !elements.is_empty() {
                    let value = if elements.len() == 1 {
                        Arc::clone(&elements[0])
                    } else if elements.iter().all(|e| Arc::ptr_eq(e, &elements[0])) {
                        Arc::clone(&elements[0])
                    } else {
                        Arc::new(Type {
                            flags: TypeFlags::Union,
                            object_flags: ObjectFlags::None,
                            id: crate::checker::types::next_type_id(),
                            symbol: None,
                            alias: None,
                            data: TypeData::Union(UnionTypeData {
                                union_or_intersection: UnionOrIntersectionTypeData {
                                    structured: StructuredTypeData::default(),
                                    types: elements,
                                },
                                resolved_reduced_type: std::sync::OnceLock::new(),
                                regular_type: std::sync::OnceLock::new(),
                                origin: None,
                                key_property_name: None,
                                constituent_map: std::collections::HashMap::new(),
                            }),
                        })
                    };
                    return Some(Arc::new(IndexInfo {
                        key_type: Some(self.number_type()),
                        value_type: Some(value),
                        is_readonly: tuple.readonly,
                        declaration: None,
                        index_symbol: None,
                        components: Vec::new(),
                    }));
                }
            }
        }
        None
    }

    pub fn get_applicable_index_info(
        &self,
        source: &Arc<Type>,
        key_type: &Arc<Type>,
    ) -> Option<Arc<IndexInfo>> {
        let infos = self.get_index_infos_of_type(source);
        for info in infos {
            if let Some(info_key) = &info.key_type {
                if Arc::ptr_eq(info_key, key_type) {
                    return Some(info);
                }

                if info_key.flags.contains(TypeFlags::Number)
                    && key_type.flags.contains(TypeFlags::String)
                {
                    return Some(info);
                }

                if info_key.flags.contains(TypeFlags::String)
                    && key_type.flags.contains(TypeFlags::Number)
                {
                    return Some(info);
                }
            }
        }
        None
    }

    pub fn is_generic_mapped_type(&self, t: &Arc<Type>) -> bool {
        if let TypeData::Mapped(m) = &t.data {
            m.type_parameter.is_some() && m.template_type.is_some()
        } else {
            false
        }
    }

    pub fn get_template_type_from_mapped_type(&self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if let TypeData::Mapped(m) = &t.data {
            return m.template_type.clone();
        }
        None
    }
}
