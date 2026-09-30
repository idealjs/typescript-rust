use std::sync::Arc;

use crate::checker::checker::Checker;
use crate::checker::mig::m2a::r19k11_defs::R19K11CheckerExt;
use crate::checker::mig::m2b::r22k6_defs;
use crate::checker::types::{
    ConditionalTypeData, IndexedAccessTypeData, LiteralTypeData, MappedTypeData, Type, TypeData,
    TypeFacts, TypeFlags,
};
use tsox_frontend::ast::{Node, Symbol};

impl Type {
    pub fn literal_data(&self) -> Option<&LiteralTypeData> {
        match &self.data {
            TypeData::Literal(d) => Some(d),
            _ => None,
        }
    }

    pub fn mapped_data(&self) -> Option<&MappedTypeData> {
        match &self.data {
            TypeData::Mapped(d) => Some(d),
            _ => None,
        }
    }

    pub fn indexed_access_data(&self) -> Option<&IndexedAccessTypeData> {
        match &self.data {
            TypeData::IndexedAccess(d) => Some(d),
            _ => None,
        }
    }

    pub fn conditional_data(&self) -> Option<&ConditionalTypeData> {
        match &self.data {
            TypeData::Conditional(d) => Some(d),
            _ => None,
        }
    }

    pub fn resolved_constraint_of_distributive(&self) -> Option<Arc<Type>> {
        self.conditional_data()
            .and_then(|d| d.resolved_constraint_of_distributive.get().cloned())
    }

    pub fn set_resolved_constraint_of_distributive(&self, t: Option<Arc<Type>>) {
        if let (TypeData::Conditional(d), Some(t)) = (&self.data, t) {
            let _ = d.resolved_constraint_of_distributive.set(t);
        }
    }

    pub fn resolved_apparent_type_of_mapped_type(&self) -> Option<Arc<Type>> {
        self.mapped_data()
            .and_then(|d| d.resolved_apparent_type.get().cloned())
    }

    pub fn set_resolved_apparent_type_of_mapped_type(&self, t: &Arc<Type>) {
        if let TypeData::Mapped(d) = &self.data {
            let _ = d.resolved_apparent_type.set(Arc::clone(t));
        }
    }

    pub fn resolved_apparent_type_of_intersection(&self) -> Option<Arc<Type>> {
        match &self.data {
            TypeData::Intersection(d) => d.resolved_apparent_type.get().cloned(),
            _ => None,
        }
    }

    pub fn set_resolved_apparent_type_of_intersection(&self, t: &Arc<Type>) {
        if let TypeData::Intersection(d) = &self.data {
            let _ = d.resolved_apparent_type.set(Arc::clone(t));
        }
    }
}

pub(crate) fn get_property_name_for_property_name_node(node: &Node) -> String {
    use tsox_frontend::ast::SyntaxKind;
    match node.kind {
        SyntaxKind::Identifier | SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral => {
            node.text().to_string()
        }
        SyntaxKind::ComputedPropertyName => node
            .expression()
            .map(|e| e.text().to_string())
            .unwrap_or_default(),
        _ => node.text().to_string(),
    }
}

impl Checker {
    pub(crate) fn get_constraint_of_type(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if t.flags.contains(TypeFlags::TypeParameter) {
            return Some(self.get_constraint_from_type_parameter(t));
        }
        self.get_base_constraint_of_type(t)
    }

    pub(crate) fn is_restrictive_instantiation(&mut self, _t: &Arc<Type>) -> bool {
        false
    }

    pub(crate) fn get_contextual_this_parameter_type(
        &mut self,
        container: &Arc<Node>,
    ) -> Option<Arc<Type>> {
        self.contextual_this_parameter_type(container)
    }

    pub(crate) fn create_declared_type_of_class_or_interface(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Arc<Type> {
        self.get_declared_type_of_symbol(symbol)
    }

    pub(crate) fn get_global_import_attributes_type(&mut self) -> Arc<Type> {
        self.unknown_type()
    }

    pub(crate) fn adjust_type_with_facts(&mut self, t: &Arc<Type>, facts: TypeFacts) -> Arc<Type> {
        let t = if self.strict_null_checks && t.flags.intersects(TypeFlags::Unknown) {
            self.unknown_union_type()
        } else {
            Arc::clone(t)
        };
        let with_facts = self.get_type_with_facts(&t, facts);
        let reduced = self.recombine_unknown_type(&with_facts);
        if self.strict_null_checks {
            if facts == TypeFacts::NE_UNDEFINED {
                return self.remove_nullable_by_intersection(
                    &reduced,
                    TypeFacts::EQ_UNDEFINED,
                    TypeFacts::EQ_NULL,
                    TypeFacts::IS_NULL,
                    &self.null_type(),
                );
            }
            if facts == TypeFacts::NE_NULL {
                return self.remove_nullable_by_intersection(
                    &reduced,
                    TypeFacts::EQ_NULL,
                    TypeFacts::EQ_UNDEFINED,
                    TypeFacts::IS_UNDEFINED,
                    &self.undefined_type(),
                );
            }
            if facts == TypeFacts::NE_UNDEFINED_OR_NULL || facts == TypeFacts::TRUTHY {
                return r22k6_defs::map_type_ext(self, &reduced, &mut |c, t| {
                    if c.has_type_facts(t, TypeFacts::EQ_UNDEFINED_OR_NULL) {
                        Some(c.get_global_non_nullable_type_instantiation(t))
                    } else {
                        Some(Arc::clone(t))
                    }
                })
                .unwrap_or_else(|| Arc::clone(&reduced));
            }
        }
        reduced
    }

    pub(crate) fn get_regular_type_of_object_type(&mut self, t: &Arc<Type>) -> Arc<Type> {
        Arc::clone(t)
    }

    pub(crate) fn distribute_conditional_type_over_facts(
        &mut self,
        t: &Arc<Type>,
        _facts: TypeFacts,
    ) -> Arc<Type> {
        Arc::clone(t)
    }
}
