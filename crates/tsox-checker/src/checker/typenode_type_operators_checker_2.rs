#![allow(unused_imports)]

use crate::checker::typenode_type_operators::*;

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

    pub(crate) fn report_missing_literal_index_property(
        &mut self,
        object_type: &Arc<Type>,
        index_type: &Arc<Type>,
        index_type_node: &Arc<Node>,
    ) {
        if !self.type_argument_stack.is_empty()
            || !index_type
                .flags
                .intersects(TypeFlags::StringLiteral | TypeFlags::NumberLiteral)
            || object_type
                .flags
                .intersects(TypeFlags::Any | TypeFlags::Unknown | TypeFlags::Never)
            || object_type.is_union()
            || object_type.is_intersection()
            || matches!(&object_type.data, TypeData::Mapped(_))
            || self.is_tuple_type(object_type)
        {
            return;
        }
        let Some(lit) = index_type.literal_value() else {
            return;
        };
        let name = match lit {
            LiteralValue::String(s) => s.clone(),
            LiteralValue::Number(n) => n.to_string(),
            _ => String::new(),
        };
        let member_missing = object_type
            .as_structured()
            .is_none_or(|s| s.members.get(&name).is_none());
        if !name.is_empty()
            && member_missing
            && self
                .get_applicable_index_info(object_type, index_type)
                .is_none()
        {
            let object_display = self.type_to_string(object_type);
            self.diagnostics.add(tsox_frontend::ast::Diagnostic::new(
                self.current_file.clone(),
                index_type_node.loc,
                tsox_core::diagnostics::messages_generated::PROPERTY_0_DOES_NOT_EXIST_ON_TYPE_1,
                vec![name, object_display],
            ));
        }
    }
}
