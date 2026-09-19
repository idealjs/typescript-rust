#![allow(unused_imports)]

use crate::checker::checker_calls::*;

// Go checkTypeArguments 的约束满足段：显式类型实参不满足（实例化后）
// 约束时报 TS2344，首错即止
impl Checker {
    pub(crate) fn check_call_type_argument_constraints(
        &mut self,
        node: &Arc<Node>,
        sig: &Arc<Signature>,
    ) {
        let type_arg_nodes: Vec<Arc<Node>> = match &node.data {
            NodeData::CallExpression(d) => d.type_arguments.as_ref(),
            NodeData::NewExpression(d) => d.type_arguments.as_ref(),
            _ => None,
        }
        .map(|l| l.iter().cloned().collect())
        .unwrap_or_default();
        if type_arg_nodes.is_empty() || sig.type_parameters.is_empty() {
            return;
        }
        let tps = sig.type_parameters.clone();
        let arg_types: Vec<Arc<Type>> = type_arg_nodes
            .iter()
            .map(|t| self.get_type_from_type_node(t))
            .collect();
        let class_subst = self.receiver_class_type_argument_substitution(node);
        for i in 0..type_arg_nodes.len().min(tps.len()) {
            let Some(constraint) = self.get_constraint_of_type_parameter(&tps[i]) else {
                continue;
            };
            let constraint =
                self.substitute_infer_type_parameters(&constraint, &tps, &arg_types);
            let constraint = match &class_subst {
                Some((class_tps, class_args)) => {
                    self.substitute_infer_type_parameters(&constraint, class_tps, class_args)
                }
                None => constraint,
            };
            let arg_type = Arc::clone(&arg_types[i]);
            if arg_type.flags.intersects(TypeFlags::Any | TypeFlags::Never)
                || self.is_error_type(&arg_type)
                || self.is_error_type(&constraint)
                || self.degraded_type_ptrs.contains(&arg_type.id)
                || self.degraded_type_ptrs.contains(&constraint.id)
            {
                continue;
            }
            if keeps_unsubstituted_type_parameter(&constraint) {
                continue;
            }
            if self.is_type_assignable_to(&arg_type, &constraint) {
                continue;
            }
            let arg_str = self.type_to_string(&arg_type);
            let constraint_str = self.type_to_string(&constraint);
            let file = self.current_file.clone();
            let mut diag = tsox_frontend::ast::Diagnostic::new(
                file,
                type_arg_nodes[i].loc,
                tsox_core::diagnostics::messages_generated::TYPE_0_DOES_NOT_SATISFY_THE_CONSTRAINT_1,
                vec![arg_str.clone(), constraint_str.clone()],
            );
            if let Some((prop, prop_src, prop_tgt)) =
                self.first_incompatible_property(&arg_type, &constraint)
            {
                let leaf = tsox_frontend::ast::Diagnostic::new(
                    None,
                    type_arg_nodes[i].loc,
                    tsox_core::diagnostics::messages_generated::TYPE_0_IS_NOT_ASSIGNABLE_TO_TYPE_1,
                    vec![prop_src, prop_tgt],
                );
                let mut prop_diag = tsox_frontend::ast::Diagnostic::new(
                    None,
                    type_arg_nodes[i].loc,
                    tsox_core::diagnostics::messages_generated::TYPES_OF_PROPERTY_0_ARE_INCOMPATIBLE,
                    vec![prop],
                );
                prop_diag.message_chain = vec![leaf];
                diag.message_chain = vec![prop_diag];
            }
            self.diagnostics.add(diag);
            break;
        }
    }

    // 方法调用位接收者的类类型实参（x.m<T> 中 x 的实例化实参），
    // 用于代入约束中残留的类类型参数
    fn receiver_class_type_argument_substitution(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<(Vec<Arc<Type>>, Vec<Arc<Type>>)> {
        let callee = match &node.data {
            NodeData::CallExpression(d) => Arc::clone(&d.expression),
            NodeData::NewExpression(d) => Arc::clone(&d.expression),
            _ => return None,
        };
        let receiver = match &callee.data {
            NodeData::PropertyAccessExpression(p) => Arc::clone(&p.expression),
            NodeData::ElementAccessExpression(e) => Arc::clone(&e.expression),
            _ => return None,
        };
        let recv_type = self.get_type_of_node(&receiver);
        let symbol = recv_type.symbol.clone()?;
        let tps = self.declared_type_parameter_types(&symbol);
        if tps.is_empty() {
            return None;
        }
        let args = match &recv_type.data {
            TypeData::Object(o) if !o.type_arguments.is_empty() => o.type_arguments.clone(),
            _ => return None,
        };
        if args.len() != tps.len() {
            return None;
        }
        Some((tps, args))
    }

    // Go checkTypeAssignableToEx 的属性不兼容 elaboration 首错属性
    fn first_incompatible_property(
        &mut self,
        source: &Arc<Type>,
        target: &Arc<Type>,
    ) -> Option<(String, String, String)> {
        let target_props = self.get_properties_of_type(target);
        let source_props = self.get_properties_of_type(source);
        for tp in target_props {
            let Some(sp) = source_props
                .iter()
                .find(|s| s.name == tp.name)
                .map(Arc::clone)
            else {
                continue;
            };
            let sp_type = self.get_type_of_symbol(&sp);
            let tp_type = self.get_type_of_symbol(&tp);
            if sp_type.flags.intersects(TypeFlags::Any)
                || tp_type.flags.intersects(TypeFlags::Any)
                || self.is_error_type(&sp_type)
                || self.is_error_type(&tp_type)
            {
                continue;
            }
            if !self.is_type_assignable_to(&sp_type, &tp_type) {
                let src_str = self.type_to_string(&sp_type);
                let tgt_str = self.type_to_string(&tp_type);
                return Some((tp.name.clone(), src_str, tgt_str));
            }
        }
        None
    }

}
pub(crate) fn keeps_unsubstituted_type_parameter(t: &Arc<Type>) -> bool {
    if t.is_type_parameter() {
        return true;
    }
    match &t.data {
        TypeData::Union(u) => u
            .union_or_intersection
            .types
            .iter()
            .any(keeps_unsubstituted_type_parameter),
        TypeData::Intersection(u) => u
            .union_or_intersection
            .types
            .iter()
            .any(keeps_unsubstituted_type_parameter),
        TypeData::Object(o) => o
            .type_arguments
            .iter()
            .any(keeps_unsubstituted_type_parameter),
        _ => false,
    }
}
