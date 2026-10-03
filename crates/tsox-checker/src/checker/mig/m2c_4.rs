#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::types_type_id::TYPE_FLAGS_STRING_OR_NUMBER_LITERAL_OR_UNIQUE;
use crate::checker::utilities_token_is_identifier_or_keyword::{
    get_property_name_from_type, is_type_usable_as_property_name,
};
use std::sync::Arc;

use super::m2c::r18k3_defs::{append_type_mapping, MappedTypeNameTypeKind};
use super::wc3::MappedTypeModifiers;
use crate::checker::Checker;
use super::wc2_2::get_mapped_type_modifiers;

impl Checker {
    pub fn resolve_mapped_type_members(&mut self, t: &Arc<Type>) { ::tsox_core::fntrace::enter("resolve_mapped_type_members"); 
        let mut members = SymbolTable::new();
        let mut index_infos: Vec<Arc<IndexInfo>> = Vec::new();
        self.set_structured_type_members(t, None, Vec::new(), Vec::new(), Vec::new());
        let type_parameter = self.get_type_parameter_from_mapped_type(t);
        let constraint_type = self.get_constraint_type_from_mapped_type(t);
        let mapped_type = t.target().cloned().unwrap_or_else(|| Arc::clone(t));
        let name_type = self.get_name_type_from_mapped_type(&mapped_type);
        let should_link_prop_declarations = self.get_mapped_type_name_type_kind(&mapped_type)
            != MappedTypeNameTypeKind::Remapping;
        let template_type = self.get_template_type_from_mapped_type(&mapped_type);
        let modifiers_source = self.get_modifiers_type_from_mapped_type(t);
        let modifiers_type = self.get_apparent_type(&modifiers_source);
        let template_modifiers = get_mapped_type_modifiers(t);
        if Checker::is_mapped_type_with_keyof_constraint_declaration(t) {
            for prop in self.get_properties_of_type(&modifiers_type) {
                let key = self.get_literal_type_from_property(&prop);
                self.add_member_for_key_type(&key, name_type.as_ref(), type_parameter.as_ref(), template_type.as_ref(), &modifiers_type, template_modifiers, should_link_prop_declarations, t, &mut members, &mut index_infos);
            }
            if modifiers_type.flags.contains(TypeFlags::ANY) {
                let key = self.string_type();
                self.add_member_for_key_type(&key, name_type.as_ref(), type_parameter.as_ref(), template_type.as_ref(), &modifiers_type, template_modifiers, should_link_prop_declarations, t, &mut members, &mut index_infos);
            } else {
                for info in self.get_index_infos_of_type(&modifiers_type) {
                    let Some(info_key_type) = info.key_type.as_ref() else { continue };
                    if info_key_type.flags.intersects(TypeFlags::String) {
                        self.add_member_for_key_type(info_key_type, name_type.as_ref(), type_parameter.as_ref(), template_type.as_ref(), &modifiers_type, template_modifiers, should_link_prop_declarations, t, &mut members, &mut index_infos);
                    }
                }
            }
        } else {
            let Some(constraint_type) = constraint_type.as_ref() else { return };
            let lower_bound = self.get_lower_bound_of_key_type(constraint_type);
            let constituents: Vec<Arc<Type>> = if lower_bound.flags.contains(TypeFlags::UNION) {
                lower_bound.types().map(|ts| ts.to_vec()).unwrap_or_default()
            } else {
                vec![Arc::clone(&lower_bound)]
            };
            for constituent in &constituents {
                self.add_member_for_key_type(constituent, name_type.as_ref(), type_parameter.as_ref(), template_type.as_ref(), &modifiers_type, template_modifiers, should_link_prop_declarations, t, &mut members, &mut index_infos);
            }
        }
        self.set_structured_type_members(t, Some(members), Vec::new(), Vec::new(), index_infos);
    }

    fn add_member_for_key_type(
        &mut self,
        key_type: &Arc<Type>,
        name_type: Option<&Arc<Type>>,
        type_parameter: Option<&Arc<Type>>,
        template_type: Option<&Arc<Type>>,
        modifiers_type: &Arc<Type>,
        template_modifiers: MappedTypeModifiers,
        should_link_prop_declarations: bool,
        t: &Arc<Type>,
        members: &mut SymbolTable,
        index_infos: &mut Vec<Arc<IndexInfo>>,
    ) {
        let prop_name_type = match name_type {
            Some(name_type) => {
                let mapping =
                    type_parameter.and_then(|tp| append_type_mapping(t.mapper(), tp, key_type));
                self.instantiate_type(name_type, mapping.as_ref())
            }
            None => Arc::clone(key_type),
        };
        let constituents: Vec<Arc<Type>> = if prop_name_type.flags.contains(TypeFlags::UNION) {
            prop_name_type
                .types()
                .map(|ts| ts.to_vec())
                .unwrap_or_default()
        } else {
            vec![Arc::clone(&prop_name_type)]
        };
        for prop_name_type in &constituents {
            self.add_member_for_key_type_worker(
                key_type,
                prop_name_type,
                type_parameter,
                template_type,
                modifiers_type,
                template_modifiers,
                should_link_prop_declarations,
                t,
                members,
                index_infos,
            );
        }
    }

    fn add_member_for_key_type_worker(
        &mut self,
        key_type: &Arc<Type>,
        prop_name_type: &Arc<Type>,
        type_parameter: Option<&Arc<Type>>,
        template_type: Option<&Arc<Type>>,
        modifiers_type: &Arc<Type>,
        template_modifiers: MappedTypeModifiers,
        should_link_prop_declarations: bool,
        t: &Arc<Type>,
        members: &mut SymbolTable,
        index_infos: &mut Vec<Arc<IndexInfo>>,
    ) {
        if is_type_usable_as_property_name(prop_name_type) {
            let prop_name = get_property_name_from_type(prop_name_type);
            if let Some(existing_prop) = members.get(&prop_name) {
                let prev_name_type = self
                    .value_symbol_links
                    .get(existing_prop)
                    .and_then(|l| l.name_type.clone());
                let name_type = match prev_name_type {
                    Some(prev) => self.get_union_type(vec![prev, Arc::clone(prop_name_type)]),
                    None => Arc::clone(prop_name_type),
                };
                if let Some(links) = self.value_symbol_links.get_mut(existing_prop) {
                    links.name_type = Some(name_type);
                }
                let prev_key_type = self
                    .mapped_symbol_links
                    .get(existing_prop)
                    .and_then(|l| l.key_type.clone());
                let existing_key_type = match prev_key_type {
                    Some(prev) => self.get_union_type(vec![prev, Arc::clone(key_type)]),
                    None => Arc::clone(key_type),
                };
                if let Some(links) = self.mapped_symbol_links.get_mut(existing_prop) {
                    links.key_type = Some(existing_key_type);
                }
            } else {
                let mut modifiers_prop: Option<Arc<Symbol>> = None;
                if is_type_usable_as_property_name(key_type) {
                    modifiers_prop = self.get_property_of_type(
                        modifiers_type,
                        &get_property_name_from_type(key_type),
                    );
                }
                let modifiers_prop_is_optional = modifiers_prop
                    .as_ref()
                    .is_some_and(|p| p.flags.intersects(SymbolFlags::Optional));
                let is_optional = template_modifiers
                    .intersects(MappedTypeModifiers::IncludeOptional)
                    || (!template_modifiers.intersects(MappedTypeModifiers::ExcludeOptional)
                        && modifiers_prop_is_optional);
                let is_readonly = template_modifiers
                    .intersects(MappedTypeModifiers::IncludeReadonly)
                    || (!template_modifiers.intersects(MappedTypeModifiers::ExcludeReadonly)
                        && modifiers_prop
                            .as_ref()
                            .is_some_and(|p| self.is_readonly_symbol(p)));
                let strip_optional = self.strict_null_checks
                    && !is_optional
                    && modifiers_prop_is_optional;
                let late_flag = modifiers_prop
                    .as_ref()
                    .map(|p| p.check_flags & CheckFlags::Late)
                    .unwrap_or(CheckFlags::empty());
                let mut prop = self.new_symbol(
                    SymbolFlags::Property
                        | if is_optional {
                            SymbolFlags::Optional
                        } else {
                            SymbolFlags::empty()
                        },
                    &prop_name,
                );
                if let Some(prop_mut) = Arc::get_mut(&mut prop) {
                    prop_mut.check_flags = late_flag
                        | CheckFlags::Mapped
                        | if is_readonly {
                            CheckFlags::Readonly
                        } else {
                            CheckFlags::empty()
                        }
                        | if strip_optional {
                            CheckFlags::StripOptional
                        } else {
                            CheckFlags::empty()
                        };
                }
                {
                    if let Some(links) = self.value_symbol_links.get_mut(&prop) {
                        links.containing_type = Some(Arc::clone(t));
                        links.name_type = Some(Arc::clone(prop_name_type));
                    }
                }
                {
                    if let Some(links) = self.mapped_symbol_links.get_mut(&prop) {
                        links.key_type = Some(Arc::clone(key_type));
                    }
                }
                if let Some(modifiers_prop) = &modifiers_prop {
                    if let Some(links) = self.mapped_symbol_links.get_mut(&prop) {
                        links.synthetic_origin = Some(Arc::clone(modifiers_prop));
                    }
                    if should_link_prop_declarations {
                        if let Some(prop_mut) = Arc::get_mut(&mut prop) {
                            prop_mut.declarations = modifiers_prop.declarations.clone();
                        }
                    }
                }
                members.insert(prop_name, prop);
            }
        } else if self.is_valid_index_key_type(prop_name_type)
            || prop_name_type
                .flags
                .intersects(TypeFlags::Any | TypeFlags::Enum)
        {
            let mut index_key_type = Arc::clone(prop_name_type);
            if prop_name_type
                .flags
                .intersects(TypeFlags::Any | TypeFlags::String)
            {
                index_key_type = self.string_type();
            } else if prop_name_type
                .flags
                .intersects(TypeFlags::Number | TypeFlags::Enum)
            {
                index_key_type = self.number_type();
            }
            let mapping =
                type_parameter.and_then(|tp| append_type_mapping(t.mapper(), tp, key_type));
            let template_type = template_type.unwrap_or(key_type);
            let prop_type = self.instantiate_type(template_type, mapping.as_ref());
            let modifiers_index_info =
                self.get_applicable_index_info(modifiers_type, prop_name_type);
            let is_readonly = template_modifiers
                .intersects(MappedTypeModifiers::IncludeReadonly)
                || (!template_modifiers.intersects(MappedTypeModifiers::ExcludeReadonly)
                    && modifiers_index_info
                        .as_ref()
                        .is_some_and(|info| info.is_readonly));
            let index_info =
                self.new_index_info(&index_key_type, &prop_type, is_readonly, None, &[]);
            *index_infos = self.append_index_info(std::mem::take(index_infos), &index_info, true);
        }
    }
}
