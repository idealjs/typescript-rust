#![allow(unused_imports)]

use std::collections::HashMap;
use std::sync::Arc;

use crate::checker::checker::Checker;
use crate::checker::mig::m1f::WideningContext;
use crate::checker::types::*;
use tsox_frontend::ast::{Node, Symbol, SymbolTable};

pub(crate) fn map_type_ex_self(
    checker: &mut Checker,
    t: &Arc<Type>,
    f: &mut dyn FnMut(&mut Checker, &Arc<Type>) -> Option<Arc<Type>>,
    no_reductions: bool,
) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("map_type_ex_self"); 
    if t.flags.contains(TypeFlags::Never) {
        return Some(Arc::clone(t));
    }
    if !t.flags.contains(TypeFlags::Union) {
        return f(checker, t);
    }
    let u = t.as_union_type();
    let mut types = u.map(|u| u.union_or_intersection.types.clone()).unwrap_or_default();
    if let Some(origin) = u.and_then(|u| u.origin.as_ref()) {
        if origin.flags.contains(TypeFlags::Union) {
            types = origin.types().unwrap_or(&[]).to_vec();
        }
    }
    let mut mapped_types: Vec<Arc<Type>> = Vec::new();
    let mut changed = false;
    for s in &types {
        let mapped = if s.flags.contains(TypeFlags::Union) {
            map_type_ex_self(checker, s, f, no_reductions)
        } else {
            f(checker, s)
        };
        if !mapped.as_ref().is_some_and(|m| Arc::ptr_eq(m, s)) {
            changed = true;
        }
        if let Some(mapped) = mapped {
            mapped_types.push(mapped);
        }
    }
    if changed {
        if mapped_types.is_empty() {
            return None;
        }
        let reduction = if no_reductions {
            UnionReduction::None
        } else {
            UnionReduction::Literal
        };
        return Some(checker.get_union_type_ex(mapped_types, reduction));
    }
    Some(Arc::clone(t))
}

impl SignatureFlags {
    pub const CallChainFlags: SignatureFlags =
        SignatureFlags::IsInnerCallChain.union(SignatureFlags::IsOuterCallChain);
}

impl Type {
    pub(crate) fn set_union_origin(&mut self, origin: Option<Arc<Type>>) { ::tsox_core::fntrace::enter("set_union_origin"); 
        if let TypeData::Union(u) = &mut self.data {
            u.origin = origin;
        }
    }

    pub(crate) fn set_alias(&mut self, alias: Option<TypeAlias>) { ::tsox_core::fntrace::enter("set_alias"); 
        self.alias = alias.map(Box::new);
    }

    pub(crate) fn set_type_parameter_target(&mut self, target: &Arc<Type>) { ::tsox_core::fntrace::enter("set_type_parameter_target"); 
        if let TypeData::TypeParameter(tp) = &mut self.data {
            tp.target = Some(Arc::clone(target));
        }
    }
}

impl WideningContext {
    pub(crate) fn with_siblings(siblings: Vec<Arc<Type>>) -> WideningContext { ::tsox_core::fntrace::enter("with_siblings"); 
        WideningContext {
            parent: None,
            property_name: String::new(),
            siblings: Some(siblings),
            resolved_properties: None,
            child_contexts: HashMap::new(),
            widened_types: HashMap::new(),
        }
    }

    pub(crate) fn get_child_context(&self, property_name: &str) -> WideningContext { ::tsox_core::fntrace::enter("get_child_context"); 
        WideningContext {
            parent: None,
            property_name: property_name.to_string(),
            siblings: None,
            resolved_properties: None,
            child_contexts: HashMap::new(),
            widened_types: HashMap::new(),
        }
    }
}

impl Checker {
    pub fn get_infer_type_parameters(&mut self, node: &Arc<Node>) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("get_infer_type_parameters"); 
        let type_parameter_symbols: Vec<Arc<Symbol>> = {
            let symbol_map = self.program.symbol_map();
            symbol_map
                .locals
                .get(&node.id())
                .map(|locals| {
                    locals
                        .entries
                        .values()
                        .filter(|s| s.flags.intersects(SymbolFlags::TypeParameter))
                        .cloned()
                        .collect()
                })
                .unwrap_or_default()
        };
        let mut result = vec![];
        for symbol in &type_parameter_symbols {
            result.push(self.get_declared_type_of_symbol(symbol));
        }
        result
    }

    pub fn get_widened_type_of_object_literal(
        &mut self,
        t: &Arc<Type>,
        context: Option<&WideningContext>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("get_widened_type_of_object_literal"); 
        let mut members = SymbolTable::new();
        for prop in self.get_properties_of_object_type(t) {
            let widened = self.get_widened_property(&prop, context);
            members.entries.insert(prop.name.clone(), widened);
        }
        let mut index_infos: Vec<Arc<IndexInfo>> = vec![];
        for info in self.get_index_infos_of_type(t) {
            if let (Some(key_type), Some(value_type)) = (&info.key_type, &info.value_type) {
                let widened_value = self.get_widened_type(value_type);
                index_infos.push(self.new_index_info(
                    key_type,
                    &widened_value,
                    info.is_readonly,
                    info.declaration.as_ref(),
                    &info.components,
                ));
            }
        }
        let symbol = t
            .symbol()
            .cloned()
            .unwrap_or_else(|| panic!("object literal type requires a symbol"));
        let mut result = self.new_anonymous_type(&symbol, members, vec![], vec![], index_infos);
        let preserved = t.object_flags & (ObjectFlags::JSLiteral | ObjectFlags::NonInferrableType);
        if let Some(result_mut) = Arc::get_mut(&mut result) {
            result_mut.object_flags.insert(preserved);
        }
        result
    }

    pub fn check_property_assignment(&mut self, node: &Arc<Node>, check_mode: CheckMode) -> Arc<Type> { ::tsox_core::fntrace::enter("check_property_assignment"); 
        if let Some(name) = node.name() {
            if tsox_frontend::ast::node_data_generated::is_computed_property_name(name) {
                self.check_computed_property_name(name);
            }
        }
        let initializer = node.initializer();
        let initializer_type = match &initializer {
            Some(init) => self.check_expression_for_mutable_location(init, check_mode),
            None => panic!("PropertyAssignment should have an initializer"),
        };
        if let Some(type_node) = node.type_() {
            let t = self.get_type_from_type_node(&type_node);
            self.check_type_assignable_to_and_optionally_elaborate(
                &initializer_type,
                &t,
                Some(node),
                initializer,
                None,
                None,
            );
            return t;
        }
        initializer_type
    }
}
