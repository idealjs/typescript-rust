#![allow(unused_imports)]

use crate::checker::relater_relate_impl_chunk::*;
use crate::checker::types::MappedTypeData;

impl Checker {
    pub(crate) fn substitute_generic_mapped_indexed_access(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("substitute_generic_mapped_indexed_access"); 
        let (constraint, name_type, type_parameter) = match &object_type.data {
            TypeData::Mapped(m) => (
                m.constraint_type.clone()?,
                m.name_type.clone(),
                m.type_parameter.clone(),
            ),
            _ => return None,
        };
        if !self.type_flags_is_generic_index_type(&constraint) {
            return None;
        }
        if let (Some(name_type), Some(type_parameter)) = (&name_type, &type_parameter)
            && !self.is_type_assignable_to(name_type, type_parameter)
        {
            return None;
        }
        let instantiated = self.instantiate_mapped_template_with_index(object_type, index_type)?;
        let is_optional = match &object_type.data {
            TypeData::Mapped(m) => self.mapped_index_access_optionality(m),
            _ => false,
        };
        Some(if is_optional {
            self.add_optionality(&instantiated)
        } else {
            instantiated
        })
    }

    fn mapped_index_access_optionality(&mut self, m: &MappedTypeData) -> bool { ::tsox_core::fntrace::enter("mapped_index_access_optionality"); 
        if Self::declared_mapped_optionality(m) > 0 {
            return true;
        }
        m.modifiers_type
            .as_ref()
            .is_some_and(|mt| self.combined_mapped_optionality(mt) > 0)
    }

    fn combined_mapped_optionality(&self, t: &Arc<Type>) -> i32 { ::tsox_core::fntrace::enter("combined_mapped_optionality"); 
        if let TypeData::Mapped(m) = &t.data {
            let optionality = Self::declared_mapped_optionality(m);
            if optionality != 0 {
                return optionality;
            }
            return m
                .modifiers_type
                .as_ref()
                .map(|mt| self.combined_mapped_optionality(mt))
                .unwrap_or(0);
        }
        if t.flags.contains(TypeFlags::Intersection)
            && let Some(constituents) = t.types()
        {
            let mut iter = constituents.iter();
            let Some(first) = iter.next() else {
                return 0;
            };
            let optionality = self.combined_mapped_optionality(first);
            let agreed = constituents
                .iter()
                .skip(1)
                .all(|c| self.combined_mapped_optionality(c) == optionality);
            return if agreed { optionality } else { 0 };
        }
        0
    }

    fn declared_mapped_optionality(m: &MappedTypeData) -> i32 { ::tsox_core::fntrace::enter("declared_mapped_optionality"); 
        m.declaration
            .as_ref()
            .and_then(|decl| match &decl.data {
                tsox_frontend::ast::NodeData::MappedTypeNode(d) => d.question_token.as_ref(),
                _ => None,
            })
            .map(|token| match token.kind {
                SyntaxKind::PlusToken => 1,
                SyntaxKind::MinusToken => -1,
                _ => 0,
            })
            .unwrap_or(0)
    }
}
