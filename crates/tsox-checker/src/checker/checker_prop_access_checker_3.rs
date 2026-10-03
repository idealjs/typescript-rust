#![allow(unused_imports)]
use crate::checker::inference::InferencePriority;

use crate::checker::checker_prop_access::*;

impl Checker {
    /// Go typeHasStaticProperty：在实例类型符号的静态侧（含继承基类静态）
    /// 命中属性且其值声明带 static 修饰
    pub(crate) fn type_has_static_property(
        &mut self,
        prop_name: &str,
        containing_type: &Arc<crate::checker::types::Type>,
    ) -> bool { ::tsox_core::fntrace::enter("type_has_static_property"); 
        let Some(sym) = containing_type.symbol.clone() else {
            return false;
        };
        let static_side = self.get_type_of_symbol(&sym);
        let Some(prop) = self.get_property_of_type(&static_side, prop_name) else {
            return false;
        };
        prop.value_declaration
            .as_ref()
            .is_some_and(|d| d.has_syntactic_modifier(ModifierFlags::Static))
    }

    pub(crate) fn global_constructor_value_has_property(
        &mut self,
        obj_expr: &Arc<Node>,
        name: &str,
    ) -> bool { ::tsox_core::fntrace::enter("global_constructor_value_has_property"); 
        if obj_expr.kind != SyntaxKind::Identifier {
            return false;
        }

        let resolved = match self.resolve_identifier(obj_expr) {
            Some(sym) => sym,
            None => return false,
        };
        let interface_name = match resolved.name.as_str() {
            "Object" => match self.globals.get("Object") {
                Some(global_sym) if Arc::ptr_eq(&resolved, global_sym) => "ObjectConstructor",
                _ => return false,
            },
            _ => return false,
        };
        self.global_interface_has_property(interface_name, name)
    }

    #[allow(dead_code)]
    pub(crate) fn property_exists_on_non_nullable_part(
        &mut self,
        t: &Arc<Type>,
        name: &str,
    ) -> bool { ::tsox_core::fntrace::enter("property_exists_on_non_nullable_part"); 
        if t.flags.contains(TypeFlags::Union) {
            if let TypeData::Union(u) = &t.data {
                for ct in &u.union_or_intersection.types {
                    if ct.flags.intersects(TypeFlags::Undefined | TypeFlags::Null) {
                        continue;
                    }
                    if self.has_property_of_type(ct, name) {
                        return true;
                    }
                }
                return false;
            }
        }

        self.has_property_of_type(t, name)
    }

    pub fn infer_call_type_arguments(
        &mut self,
        _node: &Arc<Node>,
        signature: &Arc<Signature>,
        args: &[Arc<Node>],
    ) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("infer_call_type_arguments");
        if signature.type_parameters.is_empty() {
            return Vec::new();
        }
        let inferences: Vec<InferenceInfo> = signature
            .type_parameters
            .iter()
            .map(|p| InferenceInfo::new(Arc::clone(p)))
            .collect();
        let mut context = InferenceContext::new(inferences);
        context.signature = Some(Arc::clone(signature));
        self.infer_type_arguments(_node, signature, args, &mut context)
    }

    // Go instantiateContextualType（checker.go:32609-32621）代入门槛的
    // round-1 候选池：CS 函数实参在 SkipContextSensitive 轮只产 anyFunctionType
    // 占位（checker.go:10659-10676），不把箭头自身形状喂进参数位窗口的
    // 推断池；hasInferenceCandidatesOrDefault（inference.go:1728）
    pub fn cs_excluded_inference_has_candidates(
        &mut self,
        signature: &Arc<Signature>,
        args: &[Arc<Node>],
    ) -> bool { ::tsox_core::fntrace::enter("cs_excluded_inference_has_candidates");
        if signature.type_parameters.is_empty() {
            return false;
        }
        let inferences: Vec<InferenceInfo> = signature
            .type_parameters
            .iter()
            .map(|p| InferenceInfo::new(Arc::clone(p)))
            .collect();
        let mut context = InferenceContext::new(inferences);
        context.signature = Some(Arc::clone(signature));
        let has_rest = signature.has_rest_parameter();
        let rest_index = if has_rest {
            signature.parameters.len().saturating_sub(1)
        } else {
            usize::MAX
        };
        for (i, arg) in args.iter().enumerate() {
            if matches!(
                arg.kind,
                SyntaxKind::ArrowFunction | SyntaxKind::FunctionExpression
            ) && self.is_context_sensitive(arg)
            {
                continue;
            }
            let param_type = if has_rest && i >= rest_index {
                let rest_type = self.get_type_of_symbol(&signature.parameters[rest_index]);
                self.get_array_element_type(&rest_type)
            } else if i < signature.parameters.len() {
                self.get_type_of_symbol(&signature.parameters[i])
            } else {
                continue;
            };
            if self.could_contain_type_variables(&param_type) {
                let arg_type = self.get_type_of_node(arg);
                self.infer_types(
                    &mut context.inferences,
                    Some(arg_type),
                    Some(param_type),
                    InferencePriority::None,
                    false,
                );
            }
        }
        context.inferences.iter().any(|inf| {
            !inf.candidates.is_empty() || self.type_parameter_has_default(&inf.type_parameter)
        })
    }
}
