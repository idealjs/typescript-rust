#![allow(unused_imports)]
use std::sync::Arc;

use crate::checker::checker_checker::*;
use crate::checker::types::{ElementFlags, TupleElementInfo, Type, TypeData};
use tsox_frontend::ast::node_data_generated::MappedTypeNodeData;
use tsox_frontend::ast::SymbolTable;
use tsox_frontend::ast::{Node, NodeData};

pub trait R23k4NodeExt {
    fn as_mapped_type_node(&self) -> &MappedTypeNodeData;
}

impl R23k4NodeExt for Node {
    fn as_mapped_type_node(&self) -> &MappedTypeNodeData { ::tsox_core::fntrace::enter("as_mapped_type_node"); 
        match &self.data {
            NodeData::MappedTypeNode(d) => d,
            _ => panic!("AsMappedTypeNode on wrong node kind"),
        }
    }
}

impl Checker {
    pub fn set_tuple_type_this_type(&mut self, t: &mut Arc<Type>, mut this_type: Arc<Type>) { ::tsox_core::fntrace::enter("set_tuple_type_this_type"); 
        if let Some(tp_data) = Arc::get_mut(&mut this_type).and_then(|m| match &mut m.data {
            TypeData::TypeParameter(d) => Some(d),
            _ => None,
        }) {
            tp_data.is_this_type = true;
            tp_data.constraint = Some(Arc::clone(&*t));
        }
        if let Some(t_mut) = Arc::get_mut(t) {
            if let TypeData::Tuple(d) = &mut t_mut.data {
                d.interface_data.this_type = Some(this_type);
            }
        }
    }

    pub fn get_tuple_type_this_type(&mut self, t: &Arc<Type>) -> Arc<Type> { ::tsox_core::fntrace::enter("get_tuple_type_this_type"); 
        match &t.data {
            TypeData::Tuple(d) => d
                .interface_data
                .this_type
                .clone()
                .unwrap_or_else(|| self.any_type()),
            _ => self.any_type(),
        }
    }

    pub fn set_tuple_type_data(
        &mut self,
        t: &mut Arc<Type>,
        all_type_parameters: Vec<Arc<Type>>,
        members: SymbolTable,
        element_infos: Vec<TupleElementInfo>,
        min_length: usize,
        fixed_length: usize,
        combined_flags: ElementFlags,
        readonly: bool,
    ) { ::tsox_core::fntrace::enter("set_tuple_type_data"); 
        let t_ptr: *const Type = Arc::as_ptr(t);
        let Some(t_mut) = Arc::get_mut(t) else {
            return;
        };
        let TypeData::Tuple(d) = &mut t_mut.data else {
            return;
        };
        let this_type = d.interface_data.this_type.clone();
        d.interface_data.all_type_parameters = all_type_parameters;
        d.interface_data.declared_members_resolved = true;
        d.interface_data.declared_members = members;
        unsafe {
            Arc::increment_strong_count(t_ptr);
            d.interface_data.object.target = Some(Arc::from_raw(t_ptr));
        };
        if let Some(this_type) = this_type {
            d.interface_data.object.type_arguments = d
                .interface_data
                .all_type_parameters
                .iter()
                .filter(|p| !Arc::ptr_eq(p, &this_type))
                .cloned()
                .collect();
        }
        d.element_infos = element_infos;
        d.min_length = min_length;
        d.fixed_length = fixed_length;
        d.combined_flags = combined_flags;
        d.readonly = readonly;
    }
}
