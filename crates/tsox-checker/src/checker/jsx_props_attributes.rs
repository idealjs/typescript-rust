#![allow(unused_imports)]

use crate::checker::checker::*;
use crate::checker::jsx_impl_chunk_2::*;
use crate::checker::jsx_props_check::semantic_jsx_children;
use crate::checker::types::*;
use std::sync::Arc;
use tsox_frontend::ast::{Node, NodeData, Symbol, SymbolFlags};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum JsxAttrsBuildMode {
    Full,
    SkipContextSensitive,
    GuardedPartial,
}

fn jsx_param_has_raw_type_variable(t: &Arc<Type>) -> bool { ::tsox_core::fntrace::enter("jsx_param_has_raw_type_variable"); 
    if t.flags.intersects(TypeFlags::TypeParameter) {
        return true;
    }
    match &t.data {
        TypeData::Substitution(s) => s
            .base_type
            .as_ref()
            .is_some_and(jsx_param_has_raw_type_variable),
        _ => {
            if let Some(types) = t.types() {
                return types.iter().any(jsx_param_has_raw_type_variable);
            }
            t.flags.intersects(TypeFlags::IndexedAccess | TypeFlags::Conditional)
        }
    }
}

impl Checker {
    pub(crate) fn create_jsx_attributes_type(&mut self, opening: &Arc<Node>) -> Arc<Type> { ::tsox_core::fntrace::enter("create_jsx_attributes_type"); 
        self.create_jsx_attributes_type_with_context(opening, None)
    }

    pub(crate) fn create_jsx_attributes_type_with_context(
        &mut self,
        opening: &Arc<Node>,
        context_props: Option<&Arc<Type>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("create_jsx_attributes_type_with_context"); 
        self.create_jsx_attributes_type_phased(opening, context_props, JsxAttrsBuildMode::Full)
    }

    pub(crate) fn create_jsx_attributes_type_phased(
        &mut self,
        opening: &Arc<Node>,
        context_props: Option<&Arc<Type>>,
        mode: JsxAttrsBuildMode,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("create_jsx_attributes_type_phased"); 
        let mut pairs: Vec<(String, Arc<Type>, Vec<Arc<Node>>)> = Vec::new();
        let mut spread: Option<Arc<Type>> = None;
        let children_name = self
            .get_jsx_namespace()
            .and_then(|ns| self.get_jsx_element_children_property_name(&ns));
        if let Some(attributes) = match &opening.data {
            NodeData::JsxOpeningElement(d) => Some(Arc::clone(&d.attributes)),
            NodeData::JsxSelfClosingElement(d) => Some(Arc::clone(&d.attributes)),
            _ => None,
        } {
            if let NodeData::JsxAttributes(data) = &attributes.data {
                for prop in data.properties.iter() {
                    match &prop.data {
                        NodeData::JsxAttribute(a) => {
                            let name = prop
                                .name()
                                .and_then(|n| n.jsx_namespaced_name_text())
                                .or_else(|| prop.name().map(|n| n.text().to_string()))
                                .unwrap_or_default();
                            let t = match &a.initializer {
                                Some(init) => match &init.data {
                                    NodeData::JsxExpression(e) => match &e.expression {
                                        Some(expr) => self.jsx_attr_value_type(
                                            expr,
                                            &name,
                                            context_props,
                                            mode,
                                        ),
                                        None => self.get_string_type(),
                                    },
                                    _ => self.get_type_of_node(init),
                                },
                                None => self.true_type(),
                            };
                            let t = if a.initializer.is_some() {
                                self.widen_jsx_attribute_type(&t, &name, context_props)
                            } else {
                                t
                            };
                            pairs.push((name, t, vec![Arc::clone(prop)]));
                        }
                        NodeData::JsxSpreadAttribute(a) => {
                            let expr_type = self.get_type_of_node(&a.expression);
                            let segment = self.jsx_attributes_type_from_pairs(
                                std::mem::take(&mut pairs),
                                None,
                            );
                            spread = Some(match spread {
                                Some(prev) => self.get_spread_type(
                                    &prev,
                                    &segment,
                                    None,
                                    ObjectFlags::JsxAttributes,
                                    false,
                                ),
                                None => {
                                    let empty = self.fresh_empty_object_type();
                                    self.get_spread_type(
                                        &empty,
                                        &expr_type,
                                        None,
                                        ObjectFlags::JsxAttributes,
                                        false,
                                    )
                                }
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
        let semantic_children = opening
            .parent()
            .and_then(|parent| match &parent.data {
                NodeData::JsxElement(d) => {
                    if Arc::ptr_eq(&d.opening_element, opening) {
                        Some(d.children.clone())
                    } else {
                        None
                    }
                }
                NodeData::JsxFragment(d) => {
                    if Arc::ptr_eq(&d.opening_fragment, opening) {
                        Some(d.children.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .map(|children| semantic_jsx_children(&children.nodes))
            .unwrap_or_default();
        if mode == JsxAttrsBuildMode::Full
            && !semantic_children.is_empty()
            && let Some(children_name) = children_name
            && !children_name.is_empty()
        {
            let child_types = self.jsx_child_types(&semantic_children);
            let t = if child_types.len() == 1 {
                child_types.into_iter().next().unwrap()
            } else {
                let union = self.get_union_type(child_types);
                self.create_array_type(union)
            };
            pairs.push((children_name, t, Vec::new()));
        }
        let attributes_symbol = match &opening.data {
            NodeData::JsxOpeningElement(d) => {
                self.program.symbol_map().symbol_of(&d.attributes).cloned()
            }
            NodeData::JsxSelfClosingElement(d) => {
                self.program.symbol_map().symbol_of(&d.attributes).cloned()
            }
            _ => None,
        };
        let segment = self.jsx_attributes_type_from_pairs(pairs, attributes_symbol);
        match spread {
            Some(prev) => self.get_spread_type(
                &prev,
                &segment,
                None,
                ObjectFlags::JsxAttributes,
                false,
            ),
            None => segment,
        }
    }

    fn jsx_attr_value_type(
        &mut self,
        expr: &Arc<Node>,
        name: &str,
        context_props: Option<&Arc<Type>>,
        mode: JsxAttrsBuildMode,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("jsx_attr_value_type"); 
        if mode != JsxAttrsBuildMode::Full && self.is_context_sensitive(expr) {
            if mode == JsxAttrsBuildMode::SkipContextSensitive {
                return self.any_function_type();
            }
            let Some(props) = context_props else {
                return self.any_function_type();
            };
            let Some(prop_type) = self.get_type_of_property_of_type(props, name) else {
                return self.any_function_type();
            };
            let prop_type = self.remove_missing_or_undefined_type(&prop_type);
            let prop_type = if prop_type.flags.contains(TypeFlags::Union)
                && let Some(types) = prop_type.types()
            {
                let kept: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|t| !t.flags.contains(TypeFlags::Undefined))
                    .cloned()
                    .collect();
                if kept.len() == types.len() {
                    prop_type
                } else if kept.len() == 1 {
                    Arc::clone(&kept[0])
                } else if kept.is_empty() {
                    prop_type
                } else {
                    self.get_union_type(kept)
                }
            } else {
                prop_type
            };
            let sigs = self.get_signatures_of_type(&prop_type, SignatureKind::Call);
            let Some(sig) = sigs.into_iter().next() else {
                return self.any_function_type();
            };
            let mut unresolved = false;
            for p in &sig.parameters {
                let t = self.get_type_of_symbol(p);
                if jsx_param_has_raw_type_variable(&t) {
                    unresolved = true;
                    break;
                }
            }
            if unresolved {
                return self.any_function_type();
            }
            return self.type_of_context_sensitive_arg(expr, &prop_type);
        }
        self.get_type_of_node(expr)
    }

    fn widen_jsx_attribute_type(
        &mut self,
        t: &Arc<Type>,
        name: &str,
        context_props: Option<&Arc<Type>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("widen_jsx_attribute_type"); 
        let contextual = context_props
            .and_then(|p| self.get_type_of_property_of_type(p, name));
        let literal_of_ctx = contextual
            .as_ref()
            .is_some_and(|c| self.is_literal_of_contextual_type(t, c));
        let t = if literal_of_ctx {
            Arc::clone(t)
        } else {
            self.get_widened_literal_type(t)
        };
        self.get_regular_type_of_literal_type(&t)
    }

    fn jsx_child_types(&mut self, children: &[Arc<Node>]) -> Vec<Arc<Type>> { ::tsox_core::fntrace::enter("jsx_child_types"); 
        children
            .iter()
            .map(|child| match &child.data {
                NodeData::JsxText(_) => self.get_string_type(),
                NodeData::JsxExpression(e) => match &e.expression {
                    Some(expr) => {
                        self.check_expression(expr);
                        let t = self.get_type_of_node(expr);
                        t
                    }
                    None => self.get_string_type(),
                },
                _ => {
                    self.check_expression(child);
                    self.get_type_of_node(child)
                }
            })
            .collect()
    }

    fn jsx_attributes_type_from_pairs(
        &mut self,
        prop_pairs: Vec<(String, Arc<Type>, Vec<Arc<Node>>)>,
        literal_symbol: Option<Arc<Symbol>>,
    ) -> Arc<Type> { ::tsox_core::fntrace::enter("jsx_attributes_type_from_pairs"); 
        let mut members = SymbolTable::new();
        let mut props: Vec<Arc<Symbol>> = Vec::with_capacity(prop_pairs.len());
        for (name, t, decls) in prop_pairs {
            let mut sym = Symbol::new(SymbolFlags::Property, name.clone());
            sym.value_declaration = decls.first().cloned();
            sym.declarations.extend(decls);
            let symbol = Arc::new(sym);
            self.value_symbol_links.insert(
                &symbol,
                ValueSymbolLinks {
                    resolved_type: Some(t),
                    ..Default::default()
                },
            );
            members.insert(name, Arc::clone(&symbol));
            props.push(symbol);
        }
        Arc::new(Type {
            flags: TypeFlags::Object,
            object_flags: ObjectFlags::Anonymous
                | ObjectFlags::ObjectLiteral
                | ObjectFlags::FreshLiteral
                | ObjectFlags::ContainsObjectOrArrayLiteral
                | ObjectFlags::JsxAttributes,
            id: next_type_id(),
            symbol: literal_symbol,
            alias: None,
            data: TypeData::Object(ObjectTypeData { node: None,
                structured: StructuredTypeData {
                    members,
                    properties: props,
                    ..Default::default()
                },
                ..Default::default()
            }),
        })
    }

}
