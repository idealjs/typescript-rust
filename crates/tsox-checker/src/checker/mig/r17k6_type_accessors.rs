use std::sync::Arc;

use crate::checker::types::{ConstrainedTypeData, Type, TypeData};

impl Type {
    pub fn resolved_base_constraint(&self) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("resolved_base_constraint"); 
        self.as_constrained_type()
            .and_then(|c| c.resolved_base_constraint.get().cloned())
    }

    pub fn set_resolved_base_constraint(&self, constraint: Arc<Type>) { ::tsox_core::fntrace::enter("set_resolved_base_constraint"); 
        let c = match &self.data {
            TypeData::TypeParameter(d) => Some(&d.constrained),
            TypeData::Conditional(d) => Some(&d.constrained),
            TypeData::IndexedAccess(d) => Some(&d.constrained),
            TypeData::Index(d) => Some(&d.constrained),
            _ => None,
        };
        if let Some(c) = c {
            let _ = c.resolved_base_constraint.set(constraint);
        }
    }

    pub fn type_parameter_resolved_default(&self) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("type_parameter_resolved_default"); 
        match &self.data {
            TypeData::TypeParameter(d) => d.resolved_default_type.get().cloned(),
            _ => None,
        }
    }

    pub fn set_type_parameter_resolved_default(&self, default: Arc<Type>) { ::tsox_core::fntrace::enter("set_type_parameter_resolved_default"); 
        if let TypeData::TypeParameter(d) = &self.data {
            let _ = d.resolved_default_type.set(default);
        }
    }
}
