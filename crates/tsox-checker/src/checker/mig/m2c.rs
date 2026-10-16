#![allow(unused_imports)]

#[path = "r18k3_defs.rs"]
pub mod r18k3_defs;
pub use r18k3_defs::*;
#[path = "r19k8_defs.rs"]
pub mod r19k8_defs;
pub use r19k8_defs::*;
#[path = "r21k2_defs.rs"]
pub mod r21k2_defs;
pub use r21k2_defs::*;
#[path = "r24k11_defs.rs"]
pub mod r24k11_defs;
pub use r24k11_defs::*;

use crate::checker::checker::*;
use crate::checker::mig::m2a::is_type_reference_with_generic_arguments;
use std::sync::Arc;
use tsox_frontend::ast::mig::m3f::{get_node_id, get_symbol_id};

fn write_type_reference(
    b: &mut KeyBuilder,
    ref_: &Arc<Type>,
    depth: i32,
    ignore_constraints: bool,
    constrained: &mut bool,
    type_parameters: &mut Vec<Arc<Type>>,
) { ::tsox_core::fntrace::enter("write_type_reference"); 
    if let Some(target) = ref_.target() {
        b.write_type(target);
    }
    let type_arguments: Vec<Arc<Type>> = match ref_.as_type_reference() {
        Some(data) => data.type_arguments.clone(),
        None => Vec::new(),
    };
    for t in &type_arguments {
        let is_bound = match &t.data {
            crate::checker::types_impl_chunk::TypeData::TypeParameter(tp) => {
                ignore_constraints || tp.constraint.is_none()
            }
            _ => false,
        };
        if t.flags.contains(TypeFlags::TypeParameter) {
            if is_bound {
                let index = type_parameters
                    .iter()
                    .position(|p| Arc::ptr_eq(p, t))
                    .unwrap_or_else(|| {
                        type_parameters.push(Arc::clone(t));
                        type_parameters.len() - 1
                    });
                b.write_byte(b'=');
                b.write_int(index as i32);
                continue;
            }
            *constrained = true;
        } else if depth < 4 && is_type_reference_with_generic_arguments(t) {
            b.write_byte(b'<');
            write_type_reference(b, t, depth + 1, ignore_constraints, constrained, type_parameters);
            b.write_byte(b'>');
            continue;
        }
        b.write_byte(b'-');
        b.write_type(t);
    }
}

impl KeyBuilder {
    pub fn write_string(&mut self, s: &str) { ::tsox_core::fntrace::enter("write_string"); 
        for byte in s.as_bytes() {
            self.write_byte(*byte);
        }
    }

    pub fn write_uint32(&mut self, v: u32) { ::tsox_core::fntrace::enter("write_uint32"); 
        self.write_u32(v);
    }

    pub fn write_uint64(&mut self, v: u64) { ::tsox_core::fntrace::enter("write_uint64"); 
        self.write_u64(v);
    }

    pub fn write_int(&mut self, value: i32) { ::tsox_core::fntrace::enter("write_int"); 
        self.write_u64(value as i64 as u64);
    }

    pub fn write_symbol(&mut self, s: &Arc<Symbol>) { ::tsox_core::fntrace::enter("write_symbol"); 
        self.write_u64(get_symbol_id(s));
    }

    pub fn write_generic_type_references(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
        ignore_constraints: bool,
    ) -> bool { ::tsox_core::fntrace::enter("write_generic_type_references"); 
        let mut constrained = false;
        let mut type_parameters: Vec<Arc<Type>> = Vec::with_capacity(8);
        write_type_reference(
            self,
            source,
            0,
            ignore_constraints,
            &mut constrained,
            &mut type_parameters,
        );
        self.write_byte(b',');
        write_type_reference(
            self,
            target,
            0,
            ignore_constraints,
            &mut constrained,
            &mut type_parameters,
        );
        constrained
    }

    pub fn write_node_id(&mut self, id: u64) { ::tsox_core::fntrace::enter("write_node_id"); 
        self.write_u64(id);
    }

    pub fn write_node(&mut self, node: Option<&Arc<Node>>) { ::tsox_core::fntrace::enter("write_node"); 
        if let Some(node) = node {
            let id = get_node_id(node);
            self.write_node_id(id);
        }
    }
}
