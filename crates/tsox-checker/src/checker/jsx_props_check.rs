#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::inference::{InferenceContext, InferenceInfo, InferencePriority};
use crate::checker::jsx_impl_chunk_2::*;
use crate::checker::relater_relation::RelationKind;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_frontend::ast::{Diagnostic, Node, NodeData, Symbol, SymbolFlags};

pub(crate) fn semantic_jsx_children(children: &[Arc<Node>]) -> Vec<Arc<Node>> {
    children
        .iter()
        .filter(|c| match &c.data {
            NodeData::JsxExpression(e) => e.expression.is_some(),
            NodeData::JsxText(t) => !t.contains_only_trivia_white_spaces,
            _ => true,
        })
        .cloned()
        .collect()
}

impl Checker {
    pub(crate) fn check_jsx_element_props(&mut self, opening: &Arc<Node>) {
        let attributes = match &opening.data {
            NodeData::JsxOpeningElement(d) => Some(Arc::clone(&d.attributes)),
            NodeData::JsxSelfClosingElement(d) => Some(Arc::clone(&d.attributes)),
            _ => None,
        };
        let tag_name = if opening.kind == SyntaxKind::JsxOpeningFragment {
            None
        } else {
            jsx_tag_name(opening)
        };
        // Go inferJsxTypeArguments：属性以未实例化首参型为上下文检查，
        // 类型参数约束为基元时字面量属性保值（如 T extends string 保 "x"）
        let context_props: Option<Arc<Type>> = match &tag_name {
            Some(tag) if is_jsx_intrinsic_tag_name(tag) => self.intrinsic_props_type(tag),
            Some(tag) => self.jsx_component_first_param(tag),
            None => None,
        };
        let attrs_type = self.create_jsx_attributes_type_with_context(opening, context_props.as_ref());
        let props = if opening.kind == SyntaxKind::JsxOpeningFragment {
            self.jsx_fragment_props_type(opening, &attrs_type)
        } else {
            let tag_name = match tag_name {
                Some(t) => t,
                None => return,
            };
            if is_jsx_intrinsic_tag_name(&tag_name) {
                context_props
            } else {
                self.component_props_type(opening, &tag_name, &attrs_type)
            }
        };
        let Some(props) = props else { return };
        let attrs_type = self.create_jsx_attributes_type_with_context(opening, Some(&props));
        let error_node = match jsx_tag_name(opening) {
            Some(t) => t,
            None => Arc::clone(opening),
        };
        let expr = if opening.kind == SyntaxKind::JsxOpeningFragment {
            None
        } else {
            attributes
        };
        self.check_type_related_to_and_optionally_elaborate(
            &attrs_type,
            &props,
            RelationKind::Assignable,
            Some(&error_node),
            expr.as_ref(),
            None,
            None,
        );
    }

    fn jsx_component_first_param(&mut self, tag_name: &Arc<Node>) -> Option<Arc<Type>> {
        self.check_expression(tag_name);
        let tag_type = self.get_type_of_node(tag_name);
        if tag_type.flags.contains(TypeFlags::Any) {
            return None;
        }
        let apparent = self.get_apparent_type(&tag_type);
        let construct = self.get_signatures_of_type(&apparent, SignatureKind::Construct);
        let call = self.get_signatures_of_type(&apparent, SignatureKind::Call);
        let (sig, is_class) = if !construct.is_empty() {
            (construct.into_iter().next()?, true)
        } else if !call.is_empty() {
            (call.into_iter().next()?, false)
        } else {
            let jsx_sigs = self.get_uninstantiated_jsx_signatures_of_type(&tag_type, tag_name);
            (jsx_sigs.into_iter().next()?, false)
        };
        if sig.parameters.is_empty() {
            return None;
        }
        Some(
            self.signature_instantiated_param_type(&sig, 0)
                .unwrap_or_else(|| self.get_type_of_symbol(&sig.parameters[0])),
        )
    }

    fn intersect_intrinsic_attributes(&mut self, props: Arc<Type>) -> Arc<Type> {
        if props.flags.contains(TypeFlags::Any) || self.is_error_type(&props) {
            return props;
        }
        let Some(sym) = self
            .get_jsx_type(crate::checker::jsx_impl_chunk::JsxNames::INTRINSIC_ATTRIBUTES)
        else {
            return props;
        };
        let intrinsic = self.get_declared_type_of_symbol(&sym);
        if self.is_error_type(&intrinsic) {
            return props;
        }
        self.get_intersection_type(vec![intrinsic, props])
    }

    fn intrinsic_props_type(&mut self, tag_name: &Arc<Node>) -> Option<Arc<Type>> {
        let intrinsic_elements = self.get_jsx_intrinsic_elements()?;
        let tag_text = tag_name.text().to_string();
        if let Some(member) = intrinsic_elements
            .members
            .get(&tag_text)
            .or_else(|| intrinsic_elements.exports.get(&tag_text))
        {
            let props = self.get_type_of_symbol(member);
            return Some(self.intersect_intrinsic_attributes(props));
        }
        let elements_type = self.get_type_of_symbol(&intrinsic_elements);
        for info in self.get_index_infos_of_type(&elements_type) {
            if let Some(key) = &info.key_type
                && self.is_type_assignable_to(&self.get_string_type(), key)
                && let Some(value) = &info.value_type
            {
                return Some(self.intersect_intrinsic_attributes(Arc::clone(value)));
            }
        }
        None
    }

    fn component_props_type(
        &mut self,
        opening: &Arc<Node>,
        tag_name: &Arc<Node>,
        attrs_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        self.check_expression(tag_name);
        let tag_type = self.get_type_of_node(tag_name);
        if tag_type.flags.contains(TypeFlags::Any) {
            return None;
        }
        let apparent = self.get_apparent_type(&tag_type);
        let construct = self.get_signatures_of_type(&apparent, crate::checker::types::SignatureKind::Construct);
        let call = self.get_signatures_of_type(&apparent, crate::checker::types::SignatureKind::Call);
        let (sig, is_class) = if !construct.is_empty() {
            (construct.into_iter().next()?, true)
        } else if !call.is_empty() {
            (call.into_iter().next()?, false)
        } else {
            let jsx_sigs = self.get_uninstantiated_jsx_signatures_of_type(&tag_type, &tag_name);
            (jsx_sigs.into_iter().next()?, false)
        };
        if is_class {
            self.class_props_type(opening, &sig, attrs_type)
        } else {
            let props = self.get_type_at_position(&sig, 0);
            let props = self.instantiate_jsx_props_from_attributes(opening, &sig, props, attrs_type);
            let props = self.apply_jsx_managed_attributes(opening, &props);
            Some(self.intersect_intrinsic_attributes(props))
        }
    }

    fn apply_jsx_managed_attributes(
        &mut self,
        opening: &Arc<Node>,
        props: &Arc<Type>,
    ) -> Arc<Type> {
        let Some(ns) = self.get_jsx_namespace() else {
            return Arc::clone(props);
        };
        match self.get_jsx_managed_attributes_from_located_attributes(opening, &ns, props) {
            Some(managed) => managed,
            None => Arc::clone(props),
        }
    }

    fn class_props_type(
        &mut self,
        opening: &Arc<Node>,
        sig: &Arc<crate::checker::types::Signature>,
        attrs_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let ns = self.get_jsx_namespace()?;
        let forced = self.get_name_from_jsx_element_attributes_container(
            crate::checker::jsx_impl_chunk::JsxNames::ELEMENT_ATTRIBUTES_PROPERTY_NAME_CONTAINER,
            &ns,
        );
        let instance = self.get_return_type_of_signature(sig)?;
        let props = match forced {
            None => self.get_type_at_position(sig, 0),
            Some(name) if name.is_empty() => instance.clone(),
            Some(name) => {
                let attr_type = self.get_type_of_property_of_type(&instance, &name);
                match attr_type {
                    Some(t) => t,
                    None => {
                        let has_attrs = match &opening.data {
                            NodeData::JsxOpeningElement(d) => {
                                !matches!(&d.attributes.data, NodeData::JsxAttributes(a) if a.properties.is_empty())
                            }
                            NodeData::JsxSelfClosingElement(d) => {
                                !matches!(&d.attributes.data, NodeData::JsxAttributes(a) if a.properties.is_empty())
                            }
                            _ => false,
                        };
                        if has_attrs {
                            self.grammar_error_on_node_with_args(
                                opening,
                                &tsox_core::diagnostics::messages_generated::
                                    JSX_ELEMENT_CLASS_DOES_NOT_SUPPORT_ATTRIBUTES_BECAUSE_IT_DOES_NOT_HAVE_A_0_PROPERTY,
                                &[name],
                            );
                        }
                        return None;
                    }
                }
            }
        };
        let props = self.instantiate_jsx_props_from_attributes(opening, sig, props, attrs_type);
        let props = self.apply_jsx_managed_attributes(opening, &props);
        if props.flags.contains(TypeFlags::Any) {
            return Some(props);
        }
        let mut apparent = props;
        if let Some(class_attrs_sym) =
            self.get_jsx_type(crate::checker::jsx_impl_chunk::JsxNames::INTRINSIC_CLASS_ATTRIBUTES)
        {
            let declared = self.get_declared_type_of_symbol(&class_attrs_sym);
            if !self.is_error_type(&declared) {
                let tp_count = class_attrs_sym
                    .declarations
                    .iter()
                    .filter_map(|d| match &d.data {
                        NodeData::InterfaceDeclaration(data) => {
                            data.type_parameters.as_ref().map(|tps| tps.len())
                        }
                        _ => None,
                    })
                    .max()
                    .unwrap_or(0);
                let library_attrs = if tp_count >= 1 {
                    self.resolve_interface_type_ex(&class_attrs_sym, Some(vec![instance]))
                } else {
                    declared
                };
                apparent = self.get_intersection_type(vec![library_attrs, apparent]);
            }
        }
        Some(self.intersect_intrinsic_attributes(apparent))
    }

    fn jsx_fragment_props_type(
        &mut self,
        opening: &Arc<Node>,
        attrs_type: &Arc<Type>,
    ) -> Option<Arc<Type>> {
        let frag_type = self.get_jsx_fragment_type_for_props(opening)?;
        let apparent = self.get_apparent_type(&frag_type);
        let sigs = self.get_signatures_of_type(&apparent, crate::checker::types::SignatureKind::Call);
        let sig = sigs.into_iter().next()?;
        let props = self.get_type_at_position(&sig, 0);
        let props = self.instantiate_jsx_props_from_attributes(opening, &sig, props, attrs_type);
        Some(self.intersect_intrinsic_attributes(props))
    }

    fn instantiate_jsx_props_from_attributes(
        &mut self,
        opening: &Arc<Node>,
        sig: &Arc<crate::checker::types::Signature>,
        props: Arc<Type>,
        attrs_type: &Arc<Type>,
    ) -> Arc<Type> {
        if sig.type_parameters.is_empty() || !self.could_contain_type_variables(&props) {
            return props;
        }
        let inferences: Vec<InferenceInfo> = sig
            .type_parameters
            .iter()
            .map(|p| InferenceInfo::new(Arc::clone(p)))
            .collect();
        let mut context = InferenceContext::new(inferences);
        context.signature = Some(Arc::clone(sig));
        // Go fixTypeParameters：先由跳过上下文敏感属性值的属性型固定候选，
        // 再从完整属性型推断（函数值属性的返回位候选不并入已固定参数）
        let skip = self.create_jsx_attributes_type_phased(
            opening,
            Some(&props),
            crate::checker::jsx_props_attributes::JsxAttrsBuildMode::SkipContextSensitive,
        );
        self.infer_types(
            &mut context.inferences,
            Some(skip),
            Some(Arc::clone(&props)),
            InferencePriority::None,
            false,
        );
        let pinned: Vec<Arc<Type>> = context
            .inferences
            .iter()
            .enumerate()
            .map(|(i, info)| {
                if info.candidates.is_empty() && info.contra_candidates.is_empty() {
                    Arc::clone(&info.type_parameter)
                } else {
                    self.get_inferred_type(&context, i)
                }
            })
            .collect();
        for (info, t) in context.inferences.iter_mut().zip(pinned.iter()) {
            if !info.candidates.is_empty() || !info.contra_candidates.is_empty() {
                info.is_fixed = true;
                info.inferred_type = Some(Arc::clone(t));
            }
        }
        self.infer_types(
            &mut context.inferences,
            Some(Arc::clone(attrs_type)),
            Some(Arc::clone(&props)),
            InferencePriority::None,
            false,
        );
        let inferred = self.get_inferred_types(&context);
        self.substitute_infer_type_parameters(&props, &sig.type_parameters, &inferred)
    }

    fn get_jsx_fragment_type_for_props(&mut self, opening: &Arc<Node>) -> Option<Arc<Type>> {
        use tsox_core::core::compiler_options::JsxEmit;
        let _ = opening;
        let namespace = self.jsx_mark_namespace(true);
        let should_resolve = matches!(self.compiler_options.jsx, JsxEmit::React)
            || !self.compiler_options.jsx_fragment_factory.is_empty();
        if !(should_resolve && namespace != "null") {
            return Some(self.get_any_type());
        }
        let container = self.jsx_factory_namespace_symbol(&namespace)?;
        let sym = container.exports.get("Fragment").cloned()?;
        Some(self.get_type_of_symbol(&sym))
    }

}
