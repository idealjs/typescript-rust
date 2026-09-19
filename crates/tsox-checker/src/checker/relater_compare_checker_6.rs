#![allow(unused_imports)]

use crate::checker::checker::Checker;
use crate::checker::relater_compare::*;
use crate::checker::types::{SignatureKind, Type, TypeData, TypeFlags};
use std::sync::Arc;
use tsox_frontend::ast::{Node, SyntaxKind};

impl Checker {
    pub(crate) fn is_or_has_generic_conditional(&self, t: &Arc<Type>) -> bool {
        if t.flags.contains(TypeFlags::Conditional) {
            return true;
        }
        if t.flags.contains(TypeFlags::Intersection)
            && let Some(types) = t.types()
        {
            return types.iter().any(|c| self.is_or_has_generic_conditional(c));
        }
        false
    }

    // Go elaborateDidYouMeanToCallOrConstruct：某签名返回型（非 any/never）
    // 可赋给目标时，重跑关系检查并附加「是否想调用/构造」提示
    pub(crate) fn elaborate_did_you_mean_to_call_or_construct(
        &mut self,
        node: &Arc<Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        kind: SignatureKind,
        head_message: Option<&tsox_core::diagnostics::Message>,
        out: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) -> bool {
        use tsox_core::diagnostics::messages_generated as msg;
        let signatures = self.get_signatures_of_type(source, kind);
        let has_matching_return = signatures.iter().any(|sig| {
            self.get_return_type_of_signature(sig).is_some_and(|rt| {
                !(rt.flags.contains(TypeFlags::Any) || rt.flags.contains(TypeFlags::Never))
                    && self.is_type_related_to(&rt, target, relation)
            })
        });
        if !has_matching_return {
            return false;
        }
        let mut diags: Vec<tsox_frontend::ast::Diagnostic> = Vec::new();
        if self.check_type_related_to_and_elaborate_display(
            source,
            target,
            relation,
            Some(node),
            None,
            head_message,
            Some(&mut diags),
            None,
        ) {
            return false;
        }
        let Some(mut diagnostic) = diags.into_iter().next() else {
            return false;
        };
        let hint = if kind == SignatureKind::Construct {
            msg::DID_YOU_MEAN_TO_USE_NEW_WITH_THIS_EXPRESSION
        } else {
            msg::DID_YOU_MEAN_TO_CALL_THIS_EXPRESSION
        };
        diagnostic
            .related_information
            .push(tsox_frontend::ast::Diagnostic::new(
                diagnostic.file.clone(),
                node.loc,
                hint,
                vec![],
            ));
        match out {
            Some(o) => o.push(diagnostic),
            None => self.diagnostics.add(diagnostic),
        }
        true
    }

    // Go elaborateArrowFunction：无块体、无参数注解的箭头函数，返回型不匹配时
    // 在返回表达式处细化报告
    pub(crate) fn elaborate_arrow_function(
        &mut self,
        node: &Arc<Node>,
        source: &Arc<Type>,
        target: &Arc<Type>,
        relation: RelationKind,
        mut out: Option<&mut Vec<tsox_frontend::ast::Diagnostic>>,
    ) -> bool {
        use tsox_core::diagnostics::messages_generated as msg;
        let tsox_frontend::ast::NodeData::ArrowFunction(data) = &node.data else {
            return false;
        };
        if data.body.kind == SyntaxKind::Block
            || data.parameters.iter().any(|p| {
                matches!(
                    &p.data,
                    tsox_frontend::ast::NodeData::ParameterDeclaration(pd) if pd.type_node.is_some()
                )
            })
        {
            return false;
        }
        let Some(source_sig) = self.get_single_call_signature(source) else {
            return false;
        };
        let target_signatures = self.get_signatures_of_type(target, SignatureKind::Call);
        if target_signatures.is_empty() {
            return false;
        }
        let return_expression = Arc::clone(&data.body);
        let Some(source_return) = self.get_return_type_of_signature(&source_sig) else {
            return false;
        };
        let target_returns: Vec<Arc<Type>> = target_signatures
            .iter()
            .filter_map(|s| self.get_return_type_of_signature(s))
            .collect();
        if target_returns.is_empty() {
            return false;
        }
        let target_return = self.get_union_type(target_returns);
        if self.is_type_related_to(&source_return, &target_return, relation) {
            return false;
        }
        if self.elaborate_error(
            &return_expression,
            &source_return,
            &target_return,
            relation,
            out.as_deref_mut(),
        ) {
            return true;
        }
        let mut diags: Vec<tsox_frontend::ast::Diagnostic> = Vec::new();
        self.check_type_related_to_and_elaborate_display(
            &source_return,
            &target_return,
            relation,
            Some(&return_expression),
            None,
            None,
            Some(&mut diags),
            None,
        );
        if let Some(mut diagnostic) = diags.into_iter().next() {
            if let Some(sym) = target.symbol.as_ref()
                && let Some(decl) = sym.declarations.first()
            {
                let file = self
                    .get_source_file_of_node(decl)
                    .or_else(|| self.current_file.clone());
                diagnostic
                    .related_information
                    .push(tsox_frontend::ast::Diagnostic::new(
                        file,
                        decl.loc,
                        msg::THE_EXPECTED_TYPE_COMES_FROM_THE_RETURN_TYPE_OF_THIS_SIGNATURE,
                        vec![],
                    ));
            }
            let is_async = node.has_syntactic_modifier(tsox_frontend::ast::ModifierFlags::Async);
            if !is_async
                && self
                    .get_type_of_property_of_type(&source_return, "then")
                    .is_none()
                && let Some(promise) = self.create_promise_of(&source_return)
                && self.is_type_related_to(&promise, &target_return, relation)
            {
                diagnostic
                    .related_information
                    .push(tsox_frontend::ast::Diagnostic::new(
                        diagnostic.file.clone(),
                        node.loc,
                        msg::DID_YOU_MEAN_TO_MARK_THIS_FUNCTION_AS_ASYNC,
                        vec![],
                    ));
            }
            match out {
                Some(o) => o.push(diagnostic),
                None => self.diagnostics.add(diagnostic),
            }
            return true;
        }
        false
    }

    pub(crate) fn create_promise_of(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        let promise_sym = self.globals.get("Promise")?.clone();
        if promise_sym.flags.contains(crate::checker::types::SymbolFlags::Interface) {
            Some(self.resolve_interface_type_ex(&promise_sym, Some(vec![Arc::clone(t)])))
        } else {
            None
        }
    }

    // Go getSimplifiedTypeOrConstraint：简化型（条件已解析）优先，否则约束；
    // 无约束类型参数隐含 unknown（Go getDefaultConstraintOfTypeParameter）
    pub(crate) fn get_simplified_type_or_constraint(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> {
        if t.flags.contains(TypeFlags::Conditional)
            && let Some(simplified) = self.get_resolved_type_of_conditional_type(t)
            && !Arc::ptr_eq(&simplified, t)
        {
            return Some(simplified);
        }
        if t.flags.contains(TypeFlags::TypeParameter) {
            return Some(
                self.get_constraint_of_type_parameter(t)
                    .unwrap_or_else(|| self.unknown_type()),
            );
        }
        if t.flags.contains(TypeFlags::IndexedAccess)
            || matches!(&t.data, TypeData::IndexedAccess(_))
        {
            return self.constraint_of_indexed_access(t);
        }
        None
    }
}
