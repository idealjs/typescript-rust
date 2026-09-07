#![allow(unused_imports)]

use super::*;

impl Checker {
    pub(crate) fn build_conditional_type(&mut self, node: &Arc<Node>) -> Arc<Type> {
        let (check_type_node, extends_type_node) = match &node.data {
            NodeData::ConditionalTypeNode(data) => {
                (Arc::clone(&data.check_type), Arc::clone(&data.extends_type))
            }
            _ => return self.error_type(),
        };

        let check_type = self.get_type_from_type_node(&check_type_node);
        let extends_type = self.get_type_from_type_node(&extends_type_node);

        let infer_type_parameters = self.collect_infer_type_parameters(node);

        let saved_stack = std::mem::take(&mut self.type_argument_stack);
        let saved_name_frames = std::mem::take(&mut self.type_argument_name_frames);
        let unmapped_check_type = self.get_type_from_type_node(&check_type_node);
        self.type_argument_stack = saved_stack;
        self.type_argument_name_frames = saved_name_frames;
        let is_distributive = unmapped_check_type.flags.contains(TypeFlags::TypeParameter);
        let check_type_parameter_symbol = if is_distributive {
            unmapped_check_type.symbol.clone()
        } else {
            None
        };

        let root = Box::new(ConditionalRoot {
            node: Some(Arc::clone(node)),
            check_type: Some(Arc::clone(&check_type)),
            extends_type: Some(Arc::clone(&extends_type)),
            is_distributive,
            check_type_parameter_symbol,
            infer_type_parameters: infer_type_parameters.clone(),
            outer_type_parameters: Vec::new(),
            alias: None,
            creation_scopes: self.scope_stack.clone(),
        });

        let cond_type = Arc::new(Type::new(
            TypeFlags::Conditional,
            TypeData::Conditional(ConditionalTypeData {
                constrained: ConstrainedTypeData::default(),
                root: Some(root),
                check_type: Some(Arc::clone(&check_type)),
                extends_type: Some(Arc::clone(&extends_type)),
                resolved_true_type: OnceLock::new(),
                resolved_false_type: OnceLock::new(),
                resolved_inferred_true_type: OnceLock::new(),
                resolved_default_constraint: OnceLock::new(),
                resolved_constraint_of_distributive: OnceLock::new(),
                mapper: None,
                combined_mapper: None,
                creation_type_argument_stack: self
                    .type_argument_stack
                    .iter()
                    .map(|frame| {
                        frame
                            .iter()
                            .map(|(k, v)| (*k as usize, Arc::clone(v)))
                            .collect::<HashMap<_, _>>()
                    })
                    .collect(),
            }),
        ));

        if let Some(resolved) = self.resolve_conditional_type(&cond_type) {
            resolved
        } else {
            cond_type
        }
    }

    pub(crate) fn collect_infer_type_parameters(&mut self, node: &Arc<Node>) -> Vec<Arc<Type>> {
        let symbols: Vec<Arc<Symbol>> = self
            .program
            .symbol_map()
            .locals_of(node)
            .map(|locals| {
                locals
                    .iter()
                    .filter(|(_, sym)| sym.flags.contains(SymbolFlags::TypeParameter))
                    .map(|(_, sym)| Arc::clone(sym))
                    .collect()
            })
            .unwrap_or_default();
        symbols
            .into_iter()
            .map(|sym| self.get_type_parameter_from_symbol(&sym))
            .collect()
    }

    pub(crate) fn get_type_from_import_type_node(&mut self, node: &Arc<Node>) -> Arc<Type> {
        if let Some(t) = self.get_cached_type(node) {
            return t;
        }

        if let NodeData::ImportTypeNode(d) = &node.data
            && let Some(attrs) = &d.attributes
        {
            let attrs = Arc::clone(attrs);
            let _ = self.get_resolution_mode_override(&attrs, true);
        }
        let result = self.error_type();
        self.cache_type(node, result.clone());
        result
    }

    pub(crate) fn cross_product_union_size(types: &[Arc<Type>]) -> u64 {
        let mut size: u64 = 1;
        for t in types {
            if let TypeData::Union(u) = &t.data {
                size = size.saturating_mul(u.union_or_intersection.types.len() as u64);
            } else if t.flags.contains(TypeFlags::Never) {
                return 0;
            }
        }
        size
    }
}
