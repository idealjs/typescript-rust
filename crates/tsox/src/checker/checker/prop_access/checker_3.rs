#![allow(unused_imports)]

use super::*;

impl Checker {
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

    pub(crate) fn infer_call_type_arguments(
        &mut self,
        node: &Arc<Node>,
        signature: &Arc<Signature>,
        args: &[Arc<Node>],
    ) -> Vec<Arc<Type>> {
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
        self.infer_type_arguments(node, signature, args, &mut context)
    }
}
