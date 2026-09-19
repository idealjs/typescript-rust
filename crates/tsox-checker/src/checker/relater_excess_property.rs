#![allow(unused_imports)]

use crate::checker::relater_compare::*;
use crate::checker::types::*;
use std::sync::Arc;

impl Checker {
    pub(crate) fn has_excess_properties(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
    ) -> bool {
        use tsox_core::diagnostics::messages_generated as msg;
        if !crate::checker::relater_predicates::is_excess_property_check_target(target)
            || !self.no_implicit_any && target.object_flags.contains(ObjectFlags::JSLiteral)
        {
            return false;
        }
        let is_comparing_jsx_attributes = source.object_flags.contains(ObjectFlags::JsxAttributes);
        if matches!(relation, RelationKind::Assignable | RelationKind::Comparable)
            && (self.target_admits_any_properties(target)
                || !is_comparing_jsx_attributes && self.is_empty_object_type(target))
        {
            return false;
        }
        let mut reduced_target = Arc::clone(target);
        let mut check_types: Vec<Arc<Type>> = Vec::new();
        if target.flags.contains(TypeFlags::Union)
            && let Some(members) = target.types()
        {
            let non_primitives: Vec<Arc<Type>> = members
                .iter()
                .filter(|m| m.flags.contains(TypeFlags::NonPrimitive))
                .cloned()
                .collect();
            check_types = if non_primitives.is_empty() {
                members.to_vec()
            } else {
                non_primitives
            };
            if check_types.len() == 1 {
                reduced_target = Arc::clone(&check_types[0]);
            } else if check_types.len() != members.len() {
                reduced_target = self.get_union_type(check_types.clone());
            }
        }
        let source_props = self.get_properties_of_type(source);
        let source_symbol = source.symbol.clone();
        for prop in source_props {
            let container = match &source_symbol {
                Some(s) => Arc::clone(s),
                None => break,
            };
            if !crate::checker::relater_predicates::should_check_as_excess_property(&prop, &container)
            {
                continue;
            }
            if !self.is_known_property(&reduced_target, &prop.name, is_comparing_jsx_attributes) {
                if self.relater_chain_active {
                    let error_target = self.excess_check_error_target(&reduced_target);
                    let prop_name = crate::checker::property_name_for_display(&prop.name);
                    let target_str = self.type_to_string(&error_target);
                    if let Some(decl) = prop.value_declaration.as_ref() {
                        self.relater_excess_error_node = decl.name().cloned();
                    }
                    self.relater_report_error(
                        msg::OBJECT_LITERAL_MAY_ONLY_SPECIFY_KNOWN_PROPERTIES_AND_0_DOES_NOT_EXIST_IN_TYPE_1,
                        vec![prop_name, target_str],
                    );
                }
                return true;
            }
            if !check_types.is_empty() {
                let prop_type = self.get_type_of_symbol(&prop);
                let Some(target_prop_union) =
                    self.type_of_property_in_types(&check_types, &prop.name)
                else {
                    continue;
                };
                if !self.is_type_related_to(&prop_type, &target_prop_union, relation) {
                    if self.relater_chain_active {
                        let prop_name = crate::checker::property_name_for_display(&prop.name);
                        self.relater_report_error(
                            msg::TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE,
                            vec![prop_name],
                        );
                    }
                    return true;
                }
            }
        }
        false
    }

    fn excess_check_error_target(&mut self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.intersects(TYPE_FLAGS_UNION_OR_INTERSECTION)
            && let Some(types) = t.types()
        {
            let kept: Vec<Arc<Type>> = types
                .iter()
                .filter(|m| {
                    crate::checker::relater_predicates::is_excess_property_check_target(m)
                })
                .cloned()
                .collect();
            if kept.len() == 1 {
                return Arc::clone(&kept[0]);
            }
            if kept.len() > 1 {
                return self.get_union_type(kept);
            }
        }
        Arc::clone(t)
    }

    fn type_of_property_in_types(
        &mut self,
        types: &[Arc<Type>],
        name: &str,
    ) -> Option<Arc<Type>> {
        let mut parts: Vec<Arc<Type>> = Vec::with_capacity(types.len());
        for t in types {
            parts.push(self.type_of_property_in_type(t, name));
        }
        if parts.is_empty() {
            return None;
        }
        Some(self.get_union_type(parts))
    }

    fn type_of_property_in_type(&mut self, t: &Arc<Type>, name: &str) -> Arc<Type> {
        if let Some(prop) = self.get_property_of_type(t, name) {
            return self.get_type_of_symbol(&prop);
        }
        if let Some(structured) = t.as_structured() {
            let numeric = name.parse::<f64>().is_ok();
            for info in &structured.index_infos {
                if let Some(key) = &info.key_type {
                    if key.flags.contains(TypeFlags::String)
                        || (numeric && key.flags.contains(TypeFlags::Number))
                    {
                        if let Some(v) = &info.value_type {
                            return Arc::clone(v);
                        }
                    }
                }
            }
        }
        self.undefined_type()
    }

    fn is_empty_object_type(&self, t: &Arc<Type>) -> bool {
        if !t.flags.contains(TypeFlags::Object) {
            return false;
        }
        let Some(structured) = t.as_structured() else {
            return false;
        };
        structured.properties.is_empty()
            && structured.signatures.is_empty()
            && structured.index_infos.is_empty()
    }
}
