#![allow(unused_imports)]

use crate::checker::checker_prop_access::*;

impl Checker {
    /// Go typeHasStaticProperty：在实例类型符号的静态侧（含继承基类静态）
    /// 命中属性且其值声明带 static 修饰
    pub(crate) fn type_has_static_property(
        &mut self,
        prop_name: &str,
        containing_type: &Arc<crate::checker::types::Type>,
    ) -> bool {
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
    ) -> bool {
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
    ) -> bool {
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
    ) -> Vec<Arc<Type>> {
        self.infer_call_type_arguments_with_tps(_node, signature, args).0
    }

    /// Go inferTypeArguments 后 inferredTypeParameters 随推断类型一并带回
    /// （resolveCall checker.go:9241-9245 用其重建泛型返回签名）
    pub fn infer_call_type_arguments_with_tps(
        &mut self,
        _node: &Arc<Node>,
        signature: &Arc<Signature>,
        args: &[Arc<Node>],
    ) -> (Vec<Arc<Type>>, Vec<Arc<Type>>) {
        if signature.type_parameters.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let inferences: Vec<InferenceInfo> = signature
            .type_parameters
            .iter()
            .map(|p| InferenceInfo::new(Arc::clone(p)))
            .collect();
        let mut context = InferenceContext::new(inferences);
        context.signature = Some(Arc::clone(signature));
        let types = self.infer_type_arguments(_node, signature, args, &mut context);
        let tps = std::mem::take(&mut context.inferred_type_parameters);
        (types, tps)
    }
}
