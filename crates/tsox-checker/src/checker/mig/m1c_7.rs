#![allow(unused_imports)]

#[allow(unused_imports, ambiguous_glob_reexports)]
use crate::checker::*;
#[allow(unused_imports)]
use tsox_frontend::ast::*;
#[allow(unused_imports)]
use tsox_core::diagnostics::messages_generated::*;

pub(crate) use crate::checker::checker::*;
pub(crate) use crate::checker::mig::m1c::*;
pub(crate) use crate::checker::mig::m1c_2::*;
pub(crate) use crate::checker::mig::m1c_3::*;
#[allow(unused_imports)]
use crate::checker::mig::m1g_5::get_type_list_key;
use std::sync::Arc;

use crate::checker::types::*;
use tsox_frontend::ast::{Diagnostic, Node, NodeData};

impl Checker {
    pub fn create_type_from_generic_global_type(
        &mut self,
        generic_global_type: &Arc<Type>,
        type_arguments: &[Arc<Type>],
    ) -> Arc<Type> {
        if self.empty_generic_type.get().map(|t| t.id) != Some(generic_global_type.id) {
            return self.create_type_reference(generic_global_type, type_arguments);
        }
        self.empty_object_type()
    }

    pub fn create_iterable_type(&mut self, iterated_type: &Arc<Type>) -> Arc<Type> {
        let global_iterable = self.get_global_iterable_type_checked();
        self.create_type_from_generic_global_type(
            &global_iterable,
            &[
                iterated_type.clone(),
                self.void_type(),
                self.undefined_type(),
            ],
        )
    }

    pub fn create_widening_type(&mut self, non_widening_type: &Arc<Type>) -> Arc<Type> {
        if self.strict_null_checks {
            return non_widening_type.clone();
        }
        let intrinsic_name = match &non_widening_type.data {
            TypeData::Intrinsic(data) => data.intrinsic_name.clone(),
            _ => String::new(),
        };
        let mut t = self.new_intrinsic_type(non_widening_type.flags, &intrinsic_name);
        if let Some(t_mut) = Arc::get_mut(&mut t) {
            t_mut.object_flags |= ObjectFlags::ContainsWideningType;
        }
        t
    }

    pub fn create_unknown_union_type(&mut self) -> Arc<Type> {
        if self.strict_null_checks {
            let types = vec![
                self.undefined_type(),
                self.null_type(),
                self.empty_object_type(),
            ];
            return self.get_union_type(types);
        }
        self.unknown_type()
    }

    pub fn create_type_reference(
        &mut self,
        target: &Arc<Type>,
        type_arguments: &[Arc<Type>],
    ) -> Arc<Type> {
        self.create_type_reference_ex(target, type_arguments, ObjectFlags::empty())
    }

    pub fn create_type_reference_ex(
        &mut self,
        target: &Arc<Type>,
        type_arguments: &[Arc<Type>],
        object_flags: ObjectFlags,
    ) -> Arc<Type> {
        let propagating =
            self.get_propagating_flags_of_types(type_arguments, TypeFlags::empty());
        let mut t = self.new_object_type(
            ObjectFlags::Reference | object_flags | propagating,
            target.symbol.clone(),
        );
        if let Some(t_mut) = Arc::get_mut(&mut t) {
            if let Some(obj) = t_mut.as_object_mut() {
                obj.target = Some(target.clone());
                obj.type_arguments = type_arguments.to_vec();
            }
        }
        t
    }

    pub fn create_deferred_type_reference(
        &mut self,
        target: &Arc<Type>,
        node: Option<&Arc<Node>>,
        mapper: Option<&Arc<TypeMapper>>,
        alias: Option<TypeAlias>,
    ) -> Arc<Type> {
        let alias = match alias {
            Some(a) => Some(a),
            None => {
                let mut a = node.and_then(|n| self.get_alias_for_type_node(n));
                if let (Some(a), Some(mapper)) = (&mut a, mapper) {
                    a.type_arguments = self.instantiate_types(&a.type_arguments, Some(mapper));
                }
                a
            }
        };
        let mut t = self.new_object_type(ObjectFlags::Reference, target.symbol.clone());
        if let Some(t_mut) = Arc::get_mut(&mut t) {
            let mapper = mapper.cloned();
            if let Some(obj) = t_mut.as_object_mut() {
                obj.target = Some(target.clone());
                obj.mapper = mapper;
                obj.node = node.cloned();
            }
        }
        t
    }

    pub fn clone_type_reference(&mut self, source: &Arc<Type>) -> Arc<Type> {
        let source_object = match &source.data {
            TypeData::Object(d) => Some(d),
            TypeData::Interface(i) => Some(&i.object),
            TypeData::Mapped(m) => Some(&m.object),
            _ => None,
        };
        let mut t = self.new_object_type(
            ObjectFlags::Reference,
            source.symbol.clone(),
        );
        if let Some(t_mut) = Arc::get_mut(&mut t) {
            t_mut.object_flags = source.object_flags & !ObjectFlags::MembersResolved;
            if let (Some(obj), Some(source_object)) = (t_mut.as_object_mut(), source_object) {
                obj.target = source_object.target.clone();
                obj.type_arguments = source_object.type_arguments.clone();
            }
        }
        t
    }

    pub fn extract_unit_type(&self, t: &Arc<Type>) -> Arc<Type> {
        if t.flags.contains(TypeFlags::Intersection) {
            if let TypeData::Intersection(data) = &t.data {
                if let Some(u) = data
                    .union_or_intersection
                    .types
                    .iter()
                    .find(|u| is_unit_type(u))
                {
                    return u.clone();
                }
            }
        }
        t.clone()
    }
}

pub fn find_index_info_free(
    index_infos: &[Arc<IndexInfo>],
    key_type: &Arc<Type>,
) -> Option<Arc<IndexInfo>> {
    index_infos
        .iter()
        .find(|info| info.key_type.as_ref().map(|k| k.id) == Some(key_type.id))
        .cloned()
}

