use std::sync::Arc;

use tsox_frontend::ast::{Node, Symbol};

use crate::checker::checker::*;
use crate::checker::inference::{InferenceContext, InferenceInfo, InferencePriority};

impl Checker {
    /// Go instantiateTypeWithSingleGenericCallSignature（checker.go:8064-8134）在
    /// 上下文敏感实参体返回型上的效果：Go 以 CheckMode.Contextual|Inferential
    /// 解析实参函数体返回型（checkExpressionWithContextualType checker.go:7946 →
    /// contextuallyCheckFunctionExpressionOrObjectLiteralMethod checker.go:10715 →
    /// getReturnTypeFromBody checker.go:21488），返回字面量成员或返回表达式本身
    /// 持单一泛型调用签名、上下文对应位为非泛型单一签名时，按
    /// instantiateSignatureInContextOf（checker.go:20622）外层推断语境非空臂
    /// 实例化（inferentialTypingWithFunctionTypeNested 的
    /// { x: identity } ← { x: (s: string) => U } ⇒ { x: (y: string) => string; }）。
    /// 本侧成员检查（property_assignment_type）无 CheckMode 通道，且成员型经
    /// 节点缓存供 .types 显示复用（Go 显示轮 TypeOnly 重查呈现未实例化成员，
    /// checker.go:7797/8066），故实例化只施加于签名的返回型副本
    pub(crate) fn instantiate_return_type_inferentially(
        &mut self,
        return_type: &Arc<Type>,
        contextual_signature: &Arc<Signature>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("instantiate_return_type_inferentially");
        let Some(contextual_return) = self.get_return_type_of_signature(contextual_signature)
        else {
            return Arc::clone(return_type);
        };
        if return_type.object_flags.contains(ObjectFlags::ObjectLiteral)
            && return_type.as_structured().is_some()
        {
            return self.instantiate_literal_return_members(return_type, &contextual_return);
        }
        self.instantiate_single_generic_call_signature_member(return_type, &contextual_return)
            .unwrap_or_else(|| Arc::clone(return_type))
    }

    /// Go instantiateTypeWithSingleGenericCallSignature（checker.go:8064/8134）在
    /// 非上下文敏感函数实参位的型级等价：inferTypeArguments 对参数位含类型变量
    /// 的全部实参以推断语境检查（checker.go:10009-10016
    /// checkExpressionWithContextualType 的 Inferential 位），返回字面量成员持
    /// 单一泛型调用签名、上下文对应成员为非泛型单一签名时实例化。本侧成员
    /// 检查（property_assignment_type）无 CheckMode 通道且成员型节点缓存供
    /// .types 显示复用，实例化按型施加于实参函数型的返回型副本（原字面量型
    /// 与成员符号链接不动），调用方固化节点型（对应 Go
    /// signature.resolvedReturnType 驻留，checker.go:10712/10720-10722）
    pub(crate) fn instantiate_fn_arg_return_inferentially(
        &mut self,
        arg_type: &Arc<Type>,
        contextual_type: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("instantiate_fn_arg_return_inferentially");
        let call_signature = self.single_signature_of_kind(arg_type, false)?;
        let non_nullable = self.get_non_nullable_type_of(contextual_type);
        let contextual_signature = self.single_signature_of_kind(&non_nullable, false)?;
        let contextual_return = self.get_return_type_of_signature(&contextual_signature)?;
        let arg_return = self.get_return_type_of_signature(&call_signature)?;
        if !arg_return
            .object_flags
            .contains(ObjectFlags::ObjectLiteral)
        {
            return None;
        }
        let instantiated =
            self.instantiate_literal_return_members(&arg_return, &contextual_return);
        if Arc::ptr_eq(&instantiated, &arg_return) {
            return None;
        }
        // 签名克隆换返回型（Go instantiateSignatureEx 克隆形态：字段平移，
        // resolvedReturnType 换实例化返回型）
        let mut inst = Signature::new();
        inst.flags = call_signature.flags;
        inst.min_argument_count = call_signature.min_argument_count;
        inst.resolved_min_argument_count = call_signature.resolved_min_argument_count;
        inst.declaration = call_signature.declaration.clone();
        inst.target = call_signature.target.clone();
        inst.parameters = call_signature.parameters.clone();
        inst.this_parameter = call_signature.this_parameter.clone();
        inst.type_parameters = call_signature.type_parameters.clone();
        inst.instantiated_parameter_types = call_signature.instantiated_parameter_types.clone();
        inst.resolved_type_predicate = call_signature.resolved_type_predicate.clone();
        let _ = inst.resolved_return_type.set(instantiated);
        Some(self.create_function_or_constructor_type(vec![Arc::new(inst)], false))
    }

    /// 对象字面量返回型逐成员施加实例化，嵌套字面量成员递归；任一成员变化
    /// 才重建（与 Go 逐成员 checkPropertyAssignment/checkObjectLiteralMethod
    /// 施加同型，原字面量型与成员符号链接不动）
    fn instantiate_literal_return_members(
        &mut self,
        return_type: &Arc<Type>,
        contextual_return: &Arc<Type>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("instantiate_literal_return_members");
        let props: Vec<Arc<Symbol>> = match return_type.as_structured() {
            Some(structured) => structured.properties.clone(),
            None => return Arc::clone(return_type),
        };
        let mut changed = false;
        let mut pairs: Vec<(String, Arc<Type>, Vec<Arc<Node>>)> = Vec::with_capacity(props.len());
        for prop in &props {
            let name = prop.name.clone();
            let prop_t = self.get_type_of_symbol(prop);
            let new_t = match self.get_type_of_property_of_type(contextual_return, &name) {
                Some(contextual_member) => {
                    if prop_t.object_flags.contains(ObjectFlags::ObjectLiteral)
                        && prop_t.as_structured().is_some()
                    {
                        self.instantiate_literal_return_members(&prop_t, &contextual_member)
                    } else {
                        self.instantiate_single_generic_call_signature_member(
                            &prop_t,
                            &contextual_member,
                        )
                        .unwrap_or_else(|| Arc::clone(&prop_t))
                    }
                }
                None => Arc::clone(&prop_t),
            };
            if !Arc::ptr_eq(&new_t, &prop_t) {
                changed = true;
            }
            pairs.push((name, new_t, prop.declarations.clone()));
        }
        if changed {
            let in_const_context = props
                .iter()
                .find_map(|p| p.declarations.first().and_then(|d| d.parent()))
                .map(|lit| self.is_const_context(&lit))
                .unwrap_or(false);
            return self.object_literal_type_from_pairs(pairs, return_type.symbol.clone(), in_const_context);
        }
        Arc::clone(return_type)
    }

    /// Go instantiateTypeWithSingleGenericCallSignature 守卫与尾句
    /// （checker.go:8066-8081/8134）：成员型持单一泛型调用签名、上下文成员型为
    /// 单一非泛型签名时实例化；instantiateSignatureInContextOf 外层推断语境
    /// 非空臂只做参数位推断（返回位交外层语境），rest 参数形态本侧未及（退回
    /// 原型维持现状）
    fn instantiate_single_generic_call_signature_member(
        &mut self,
        t: &Arc<Type>,
        contextual_member: &Arc<Type>,
    ) -> Option<Arc<Type>> { ::tsox_core::fntrace::enter("instantiate_single_generic_call_signature_member");
        let call_signature = self.get_single_signature(t, SignatureKind::Call, true);
        let is_call = call_signature.is_some();
        let signature =
            call_signature.or_else(|| self.get_single_signature(t, SignatureKind::Construct, true))?;
        if signature.type_parameters.is_empty() {
            return None;
        }
        let kind = if is_call {
            SignatureKind::Call
        } else {
            SignatureKind::Construct
        };
        let non_nullable = self.get_non_nullable_type(contextual_member);
        let contextual_signature = self.get_single_signature(&non_nullable, kind, false)?;
        if !contextual_signature.type_parameters.is_empty() {
            return None;
        }
        if self.get_effective_rest_type(&contextual_signature).is_some()
            || self.get_effective_rest_type(&signature).is_some()
        {
            return None;
        }
        let inferences: Vec<InferenceInfo> = signature
            .type_parameters
            .iter()
            .map(|p| InferenceInfo::new(Arc::clone(p)))
            .collect();
        let mut context = InferenceContext::new(inferences);
        context.signature = Some(Arc::clone(&signature));
        if let (Some(contextual_this), Some(source_this)) = (
            self.get_this_type_of_signature(&contextual_signature),
            self.get_this_type_of_signature(&signature),
        ) {
            self.infer_types(
                &mut context.inferences,
                Some(contextual_this),
                Some(source_this),
                InferencePriority::None,
                false,
            );
        }
        let param_count = self
            .get_parameter_count(&contextual_signature)
            .min(self.get_parameter_count(&signature));
        for i in 0..param_count {
            let source = self.get_type_at_position(&contextual_signature, i);
            let target = self.get_type_at_position(&signature, i);
            self.infer_types(
                &mut context.inferences,
                Some(source),
                Some(target),
                InferencePriority::None,
                false,
            );
        }
        let inferred = self.get_inferred_types(&mut context);
        let instantiated = self.get_signature_instantiation(&signature, &inferred, &[]);
        Some(self.get_or_create_type_from_signature(&instantiated))
    }
}
