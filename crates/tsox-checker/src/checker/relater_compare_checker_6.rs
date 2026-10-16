#![allow(unused_imports)]

use crate::checker::checker::Checker;
use crate::checker::relater_compare::*;
use crate::checker::types::{IndexFlags, SignatureKind, Type, TypeData, TypeFlags};
use std::sync::Arc;
use tsox_frontend::ast::{Node, SyntaxKind};

impl Checker {
    pub(crate) fn is_or_has_generic_conditional(&self, t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("is_or_has_generic_conditional"); 
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
    ) -> bool { ::tsox_core::fntrace::enter("elaborate_did_you_mean_to_call_or_construct"); 
        use tsox_core::diagnostics::messages_generated as msg;
        let signatures = self.get_signatures_of_type(source, kind);
        let saved_chain = std::mem::take(&mut self.relater_error_chain);
        let was_active = self.relater_chain_active;
        self.relater_chain_active = false;
        let has_matching_return = signatures.iter().any(|sig| {
            self.get_return_type_of_signature(sig).is_some_and(|rt| {
                !(rt.flags.contains(TypeFlags::Any) || rt.flags.contains(TypeFlags::Never))
                    && self.is_type_related_to(&rt, target, relation)
            })
        });
        self.relater_error_chain = saved_chain;
        self.relater_chain_active = was_active;
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
    ) -> bool { ::tsox_core::fntrace::enter("elaborate_arrow_function"); 
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
        let saved_chain_active = self.relater_chain_active;
        self.relater_chain_active = false;
        let related = self.is_type_related_to(&source_return, &target_return, relation);
        self.relater_chain_active = saved_chain_active;
        if related {
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
            {
                let saved_chain_active = self.relater_chain_active;
                self.relater_chain_active = false;
                let promise_related = self.is_type_related_to(&promise, &target_return, relation);
                self.relater_chain_active = saved_chain_active;
                if promise_related {
                    diagnostic
                        .related_information
                        .push(tsox_frontend::ast::Diagnostic::new(
                            diagnostic.file.clone(),
                            node.loc,
                            msg::DID_YOU_MEAN_TO_MARK_THIS_FUNCTION_AS_ASYNC,
                            vec![],
                        ));
                }
            }
            match out {
                Some(o) => o.push(diagnostic),
                None => self.diagnostics.add(diagnostic),
            }
            return true;
        }
        false
    }

    pub(crate) fn create_promise_of(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("create_promise_of"); 
        let promise_sym = self.globals.get("Promise")?.clone();
        if promise_sym.flags.contains(crate::checker::types::SymbolFlags::Interface) {
            Some(self.resolve_interface_type_ex(&promise_sym, Some(vec![Arc::clone(t)])))
        } else {
            None
        }
    }

    // Go getSimplifiedTypeOrConstraint（checker.go:29694-29700）：简化型优先
    //（getSimplifiedType 对 IndexedAccess 经 getSimplifiedIndexedAccessType
    // checker.go:29586-29630 的 keyof 联合索引分发 distributeObjectOverIndexType
    // checker.go:29632-29644、对象联合/交积分发、泛型映射代入；对 Conditional
    // 经 forced branch 归约），无简化落 getConstraintOfType（checker.go:
    // 18040-18050）按型取约束。此前序是 keyof T 键检查的通关路径：
    // T extends TLShape 的 TLShape = TLIndexedShapes[keyof TLIndexedShapes]
    // 经 index 联合分发直接得值联合（constraint 侧 index/object 双 miss：
    // keyof 具体映射作索引用返回 nil checker.go:18240-18244、mapped 型无
    // 基约束 checker.go:29066 宽门不含 Object 位），r2 前直接查约束跳过分发
    // 使链条断在 None。类型参数约束经 getConstraintFromTypeParameter 字段
    // miss 时声明回源（checker.go:18074-18099），无约束返回 nil——本仓类型
    // 参数实例双缓存（declared_type_links 构造不带约束解析），裸读字段 None
    // 后兜底 unknown 是自造语义：keyof unknown = never（组合已对齐 Go
    // getIndexTypeEx checker.go:28290-28291）使 T[K] 键检查键集恒空误报
    // TS2536，改声明回源，无约束才落空（relater.go:3698 constraint!=nil
    // 才展开）
    pub(crate) fn get_simplified_type_or_constraint(&mut self, t: &Arc<Type>) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("get_simplified_type_or_constraint"); 
        let simplified = self.get_simplified_type(t, false);
        if !Arc::ptr_eq(&simplified, t) {
            return Some(simplified);
        }
        if t.flags.contains(TypeFlags::TypeParameter) {
            if let Some(c) = self.get_constraint_of_type_parameter(t) {
                return Some(c);
            }
            let constraint_declaration = t.symbol.as_ref().and_then(|sym| {
                sym.declarations.iter().find_map(|d| {
                    match &d.data {
                        tsox_frontend::ast::NodeData::TypeParameterDeclaration(td) => {
                            td.constraint.clone()
                        }
                        _ => None,
                    }
                })
            });
            if let Some(cn) = constraint_declaration {
                return Some(self.get_type_from_type_node(&cn));
            }
            return None;
        }
        if t.flags.contains(TypeFlags::Index) || matches!(&t.data, TypeData::Index(_)) {
            // Go getConstraintOfType→getBaseConstraintOfType（checker.go:29066
            // Index 位命中宽门）→ computeBaseConstraint Index 臂（checker.go:
            // 29154-29161）：泛型映射目标带 as 名型（非 keyof 声明约束）时键集
            // 先经 getIndexTypeForMappedType 再取下一层基约束，否则 keyof 型
            // 的基约束 = string|number|symbol；缺此臂使 keyof 具体映射（如
            // TLIndexedShapes）作索引用时 index 侧约束恒落空，A[K] 的约束
            // 链无法经 index 侧展开（Go checker.go:18238-18244）
            if let TypeData::Index(it) = &t.data
                && let Some(target) = &it.target
                && self.mapped_type_keys_are_generic(target)
            {
                let name_typed = self.get_name_type_from_mapped_type(target).is_some()
                    && !Self::is_mapped_type_with_keyof_constraint_declaration(target);
                if name_typed {
                    let keys = self.get_index_type_for_mapped_type(target, IndexFlags::None);
                    if let Some(next) = self.get_simplified_type_or_constraint(&keys) {
                        return Some(next);
                    }
                }
            }
            let parts = vec![
                self.string_type(),
                self.number_type(),
                self.es_symbol_type(),
            ];
            return Some(self.get_union_type(parts));
        }
        if t.flags.contains(TypeFlags::IndexedAccess)
            || matches!(&t.data, TypeData::IndexedAccess(_))
        {
            // Go getConstraintOfIndexedAccess（checker.go:18222-18228）→
            // getConstraintFromIndexedAccess（checker.go:18230-18250）：泛型
            // 映射索引代入 → index 侧约束索引 → object 侧约束索引三段，
            // m1d_2 get_constraint_from_indexed_access 为同构实现
            return self.get_constraint_from_indexed_access(t);
        }
        if t.flags.contains(TypeFlags::Conditional) {
            // Go getConstraintOfType Conditional 位（checker.go:18044-18046）→
            // getConstraintOfConditionalType（checker.go:18260-18267）：分布约束
            //（实例化归约 never 放弃）优先，否则 default（true|false 分支约束
            // 并集），m1d_2 get_constraint_from_conditional_type 为同构实现
            return self.get_constraint_from_conditional_type(t);
        }
        None
    }
}
