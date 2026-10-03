use std::sync::{Arc, OnceLock};

use tsox_frontend::ast::{Node, Symbol, SymbolFlags, SyntaxKind};

use crate::checker::checker::Checker;
use crate::checker::mig::m1e::R20k2NodeExt;
use crate::checker::types::{
    AccessFlags, ConditionalTypeData, ConstrainedTypeData, EvolvingArrayTypeData, IndexFlags,
    IndexedAccessTypeData, IndexTypeData, InstantiationExpressionTypeData, IntrinsicTypeData,
    LiteralTypeData, LiteralValue, MappedTypeData, ObjectTypeData, ReverseMappedTypeData,
    StringMappingTypeData, SubstitutionTypeData, TemplateLiteralTypeData,
};

pub struct PatternAmbientModule {
    pub pattern: tsox_core::core::core::Pattern,
    pub symbol: Arc<Symbol>,
}

pub fn empty_conditional_type_data() -> ConditionalTypeData { ::tsox_core::fntrace::enter("empty_conditional_type_data"); 
    ConditionalTypeData {
        constrained: ConstrainedTypeData::default(),
        root: None,
        check_type: None,
        extends_type: None,
        resolved_true_type: OnceLock::new(),
        resolved_false_type: OnceLock::new(),
        resolved_inferred_true_type: OnceLock::new(),
        resolved_default_constraint: OnceLock::new(),
        resolved_constraint_of_distributive: OnceLock::new(),
        mapper: None,
        combined_mapper: None,
        creation_type_argument_stack: Vec::new(),
    }
}

pub fn empty_index_type_data() -> IndexTypeData { ::tsox_core::fntrace::enter("empty_index_type_data"); 
    IndexTypeData {
        constrained: ConstrainedTypeData::default(),
        target: None,
        index_flags: IndexFlags::empty(),
    }
}

pub fn empty_indexed_access_type_data() -> IndexedAccessTypeData { ::tsox_core::fntrace::enter("empty_indexed_access_type_data"); 
    IndexedAccessTypeData {
        constrained: ConstrainedTypeData::default(),
        object_type: None,
        index_type: None,
        access_flags: AccessFlags::empty(),
    }
}

pub fn empty_literal_type_data(value: LiteralValue) -> LiteralTypeData { ::tsox_core::fntrace::enter("empty_literal_type_data"); 
    LiteralTypeData {
        value,
        fresh_type: OnceLock::new(),
        regular_type: OnceLock::new(),
    }
}

pub fn empty_intrinsic_type_data() -> IntrinsicTypeData { ::tsox_core::fntrace::enter("empty_intrinsic_type_data"); 
    IntrinsicTypeData {
        intrinsic_name: String::new(),
    }
}

pub fn empty_mapped_type_data() -> MappedTypeData { ::tsox_core::fntrace::enter("empty_mapped_type_data"); 
    MappedTypeData {
        object: ObjectTypeData::default(),
        declaration: None,
        type_parameter: None,
        constraint_type: None,
        name_type: None,
        template_type: None,
        template_node: None,
        template_subst: None,
        modifiers_type: None,
        resolved_apparent_type: OnceLock::new(),
        contains_error: std::sync::atomic::AtomicBool::new(false),
    }
}

pub fn empty_reverse_mapped_type_data() -> ReverseMappedTypeData { ::tsox_core::fntrace::enter("empty_reverse_mapped_type_data"); 
    ReverseMappedTypeData {
        object: ObjectTypeData::default(),
        source: None,
        mapped_type: None,
        constraint_type: None,
    }
}

pub fn empty_evolving_array_type_data() -> EvolvingArrayTypeData { ::tsox_core::fntrace::enter("empty_evolving_array_type_data"); 
    EvolvingArrayTypeData {
        object: ObjectTypeData::default(),
        element_type: None,
        final_array_type: OnceLock::new(),
    }
}

pub fn empty_instantiation_expression_type_data() -> InstantiationExpressionTypeData { ::tsox_core::fntrace::enter("empty_instantiation_expression_type_data"); 
    InstantiationExpressionTypeData {
        object: ObjectTypeData::default(),
        node: None,
    }
}

pub fn empty_string_mapping_type_data() -> StringMappingTypeData { ::tsox_core::fntrace::enter("empty_string_mapping_type_data"); 
    StringMappingTypeData {
        constrained: ConstrainedTypeData::default(),
        target: None,
    }
}

pub fn empty_substitution_type_data() -> SubstitutionTypeData { ::tsox_core::fntrace::enter("empty_substitution_type_data"); 
    SubstitutionTypeData {
        constrained: ConstrainedTypeData::default(),
        base_type: None,
        constraint: None,
    }
}

pub fn empty_template_literal_type_data() -> TemplateLiteralTypeData { ::tsox_core::fntrace::enter("empty_template_literal_type_data"); 
    TemplateLiteralTypeData {
        constrained: ConstrainedTypeData::default(),
        texts: Vec::new(),
        types: Vec::new(),
    }
}

pub fn get_type_reference_name_arc(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_type_reference_name_arc"); 
    match node.kind {
        SyntaxKind::TypeReference => {
            Some(Arc::clone(&node.as_type_reference_node().type_name))
        }
        SyntaxKind::ExpressionWithTypeArguments => {
            let expr = node.expression()?;
            if tsox_frontend::ast::is_entity_name_expression(expr) {
                Some(Arc::clone(expr))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub trait R20K6CheckerExt {
    fn resolve_type_reference_name(
        &mut self,
        type_reference: &Arc<Node>,
        meaning: SymbolFlags,
        ignore_errors: bool,
    ) -> Option<Arc<Symbol>>;
}

impl R20K6CheckerExt for Checker {
    fn resolve_type_reference_name(
        &mut self,
        type_reference: &Arc<Node>,
        meaning: SymbolFlags,
        ignore_errors: bool,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("resolve_type_reference_name"); 
        let Some(name) = get_type_reference_name_arc(type_reference) else {
            return Some(self.unknown_symbol());
        };
        let symbol = self.resolve_entity_name(&name, meaning, ignore_errors, false, None);
        if let Some(symbol) = symbol {
            if !Arc::ptr_eq(&symbol, &self.unknown_symbol()) {
                return Some(symbol);
            }
        }
        if ignore_errors {
            return Some(self.unknown_symbol());
        }
        self.get_unresolved_symbol_for_entity_name(&name)
    }
}
