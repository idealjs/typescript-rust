#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_core::core::compiler_options::{CompilerOptions, JsxEmit, ScriptTarget};
use tsox_core::core::text::TextRange;
use tsox_core::stringutil::{
    compare_strings_case_sensitive, encode_js_string_rune, is_line_break,
    is_white_space_single_line,
};
use tsox_frontend::ast::node::{Node, NodeList, SourceFile};
use tsox_frontend::ast::node_data_generated::{NodeData, TokenFlags};
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::node_source_file::LanguageVariant;
use tsox_frontend::ast::subtree_facts::{SubtreeContainsJsx, SubtreeFacts};
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::utilities::*;
use tsox_frontend::ast::{node_data_generated::*, *};
use tsox_frontend::astnav::get_line_and_character_of_position
    as get_ecma_line_and_utf16_character_of_position;
use tsox_frontend::scanner::TOKEN_FLAGS_NONE;
use tsox_frontend::scanner::mig::m3i::{is_identifier_text, is_intrinsic_jsx_name};
use tsox_frontend::scanner::skip_trivia;
use tsox_checker::checker::mig::m2d::EmitResolver;
use tsox_frontend::format::mig::m4o::EmitFlags;

use crate::printer::generated_identifier_flags::{
    AutoGenerateOptions, GeneratedIdentifierFlags,
};
use crate::printer::{EmitContext, NodeFactory};

pub trait R40K08EmitResolverExt {
    fn get_referenced_export_container_r40k08(
        &self,
        node: &Arc<Node>,
        prefix_locals: bool,
    ) -> Option<Arc<Node>>;
    fn get_jsx_factory_entity_r40k08(&self, location: &Arc<Node>) -> Option<Arc<Node>>;
    fn get_jsx_fragment_factory_entity_r40k08(&self, location: &Arc<Node>) -> Option<Arc<Node>>;
}

impl R40K08EmitResolverExt for EmitResolver {
    fn get_referenced_export_container_r40k08(
        &self,
        _node: &Arc<Node>,
        _prefix_locals: bool,
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_referenced_export_container_r40k08"); 
        None
    }

    fn get_jsx_factory_entity_r40k08(&self, _location: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_jsx_factory_entity_r40k08"); 
        None
    }

    fn get_jsx_fragment_factory_entity_r40k08(&self, _location: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_jsx_fragment_factory_entity_r40k08"); 
        None
    }
}

#[path = "r33k11_defs.rs"]
pub mod r33k11_defs;
use crate::mig::m4j::r33k11_defs::{
    create_expression_from_entity_name, get_jsx_implicit_import_base, get_jsx_runtime_import,
    get_semantic_jsx_children, is_jsx_opening_like_element, set_parent_in_children,
};

#[path = "r36k3_defs.rs"]
pub mod r36k3_defs;
use crate::mig::m4j::r36k3_defs::{
    JsxNodeVisitor, R36K3NodeAccessExt, R36K3NodeFactoryExt, R36K3SourceFileExt,
};

#[path = "r39k03_defs.rs"]
pub mod r39k03_defs;
use crate::mig::m4j::r39k03_defs::R39K03EmitResolverExt;

pub struct JsxTransformer {
    pub compiler_options: Arc<CompilerOptions>,
    pub emit_resolver: Arc<EmitResolver>,
    pub import_specifier: String,
    pub filename_declaration: Option<Arc<Node>>,
    pub utilized_implicit_runtime_imports: Vec<(String, HashMap<String, Arc<Node>>)>,
    pub in_jsx_child: bool,
    pub current_source_file: Option<Arc<SourceFile>>,
    pub emit_context: EmitContext,
}

impl JsxTransformer {
    fn factory(&self) -> NodeFactory<'_> { ::tsox_core::fntrace::enter("factory"); 
        NodeFactory::new(&self.emit_context)
    }

    fn emit_context(&mut self) -> &mut EmitContext { ::tsox_core::fntrace::enter("emit_context"); 
        &mut self.emit_context
    }

    fn visitor(&mut self) -> JsxNodeVisitor<'_> { ::tsox_core::fntrace::enter("visitor"); 
        JsxNodeVisitor { tx: self }
    }

    fn utilized_imports_get(&self, import_source: &str) -> Option<&HashMap<String, Arc<Node>>> { ::tsox_core::fntrace::enter("utilized_imports_get"); 
        self.utilized_implicit_runtime_imports
            .iter()
            .find(|(k, _)| k == import_source)
            .map(|(_, v)| v)
    }

    fn utilized_imports_entry_mut(&mut self, import_source: &str) -> &mut HashMap<String, Arc<Node>> { ::tsox_core::fntrace::enter("utilized_imports_entry_mut"); 
        if let Some(idx) =
            self.utilized_implicit_runtime_imports.iter().position(|(k, _)| k == import_source)
        {
            return &mut self.utilized_implicit_runtime_imports[idx].1;
        }
        self.utilized_implicit_runtime_imports
            .push((import_source.to_string(), HashMap::new()));
        let last = self.utilized_implicit_runtime_imports.len() - 1;
        &mut self.utilized_implicit_runtime_imports[last].1
    }

    pub fn get_current_file_name_expression(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("get_current_file_name_expression"); 
        if let Some(d) = &self.filename_declaration {
            return d.as_variable_declaration().name.clone();
        }
        let name = self.factory().new_unique_name_node(
            "_jsxFileName",
            AutoGenerateOptions {
                flags: GeneratedIdentifierFlags::OPTIMISTIC
                    | GeneratedIdentifierFlags::FILE_LEVEL,
                ..Default::default()
            },
        );
        let literal = self.factory().new_string_literal(
            self.current_source_file.as_ref().unwrap().file_name(),
            TOKEN_FLAGS_NONE,
        );
        let d = self.factory().new_variable_declaration(&name, None, None, Some(&literal));
        self.filename_declaration = Some(d.clone());
        d.as_variable_declaration().name.clone()
    }

    pub fn get_jsx_factory_callee_primitive(&self, is_static_children: bool) -> &'static str { ::tsox_core::fntrace::enter("get_jsx_factory_callee_primitive"); 
        if self.compiler_options.jsx == JsxEmit::ReactJSXDev {
            return "jsxDEV";
        }
        if is_static_children {
            return "jsxs";
        }
        "jsx"
    }

    pub fn get_jsx_factory_callee(&mut self, is_static_children: bool) -> Arc<Node> { ::tsox_core::fntrace::enter("get_jsx_factory_callee"); 
        let t = self.get_jsx_factory_callee_primitive(is_static_children);
        self.get_implicit_import_for_name(t)
    }

    pub fn get_implicit_jsx_fragment_reference(&mut self) -> Arc<Node> { ::tsox_core::fntrace::enter("get_implicit_jsx_fragment_reference"); 
        self.get_implicit_import_for_name("Fragment")
    }

    pub fn get_implicit_import_for_name(&mut self, name: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("get_implicit_import_for_name"); 
        let import_source = self.import_specifier.clone();
        let import_source = if name != "createElement" {
            get_jsx_runtime_import(&import_source, &self.compiler_options)
        } else {
            import_source
        };
        if let Some(existing) = self.utilized_imports_get(&import_source) {
            if let Some(elem) = existing.get(name) {
                return elem.as_import_specifier().name.clone();
            }
        } else {
            self.utilized_imports_entry_mut(&import_source);
        }

        let generated_name = self.factory().new_unique_name_node(
            &format!("_{}", name),
            AutoGenerateOptions {
                flags: GeneratedIdentifierFlags::OPTIMISTIC
                    | GeneratedIdentifierFlags::FILE_LEVEL
                    | GeneratedIdentifierFlags::ALLOW_NAME_SUBSTITUTION,
                ..Default::default()
            },
        );
        let specifier = self.factory().new_import_specifier(
            false,
            Some(self.factory().new_identifier(name)),
            generated_name,
        );
        self.emit_resolver.set_referenced_import_declaration(
            &specifier.as_import_specifier().name,
            Some(specifier.clone()),
        );
        self.utilized_imports_entry_mut(&import_source)
            .insert(name.to_string(), specifier.clone());
        specifier.as_import_specifier().name.clone()
    }

    pub fn set_in_child(&mut self, v: bool) { ::tsox_core::fntrace::enter("set_in_child"); 
        self.in_jsx_child = v;
    }

    pub fn visit(&mut self, node: Option<&Arc<Node>>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit"); 
        let node = node?;
        if node.subtree_facts().intersects(SubtreeContainsJsx) {
            return Some(node.clone());
        }
        match node.kind {
            SyntaxKind::SourceFile => {
                panic!("nested SourceFile should be visited via visit_source_file entry")
            }
            SyntaxKind::JsxElement => Some(self.visit_jsx_element(node)),
            SyntaxKind::JsxSelfClosingElement => Some(self.visit_jsx_self_closing_element(node)),
            SyntaxKind::JsxFragment => Some(self.visit_jsx_fragment(node)),
            SyntaxKind::JsxOpeningElement => {
                panic!("JsxOpeningElement should not be visited, handled in visit_jsx_element")
            }
            SyntaxKind::JsxOpeningFragment => {
                panic!("JsxOpeningFragment should not be visited, handled in visit_jsx_fragment")
            }
            SyntaxKind::JsxText => {
                self.set_in_child(false);
                self.visit_jsx_text(node)
            }
            SyntaxKind::JsxExpression => {
                self.set_in_child(false);
                self.visit_jsx_expression(node)
            }
            _ => {
                self.set_in_child(false);
                self.visitor().visit_each_child(node)
            }
        }
    }

    pub fn visit_source_file(&mut self, file: &Arc<SourceFile>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_source_file"); 
        if file.is_declaration_file {
            return file.as_node();
        }

        crate::mig::m4m_2::register_source_file_line_map(file);
        self.current_source_file = Some(file.clone());
        self.import_specifier =
            get_jsx_implicit_import_base(&self.compiler_options, file);
        self.filename_declaration = None;
        self.utilized_implicit_runtime_imports.clear();

        let visited = self.visitor().visit_each_child(&file.as_node()).unwrap();
        let helpers = self.emit_context().read_emit_helpers();
        self.emit_context().add_emit_helper(&visited, &helpers);
        let mut statements = visited.statements();
        let mut statements_updated = false;
        if let Some(filename_declaration) = self.filename_declaration.clone() {
            let declaration_statement = self.factory().new_variable_statement(
                None,
                &self.factory().new_variable_declaration_list(
                    &self.factory().new_node_list(vec![filename_declaration]),
                    NodeFlags::Const,
                ),
            );
            statements =
                self.insert_statement_after_custom_prologue(statements, Some(declaration_statement));
            statements_updated = true;
        }

        if !self.utilized_implicit_runtime_imports.is_empty() {
            if is_external_module(file) {
                statements_updated = true;
                let mut new_statements = Vec::new();
                for (import_source, import_specifiers_map) in
                    self.utilized_implicit_runtime_imports.clone()
                {
                    let s = self.factory().new_import_declaration(
                        None,
                        Some(self.factory().new_import_clause(
                            SyntaxKind::Unknown,
                            None,
                            Some(self.factory().new_named_imports(self.factory().new_node_list(
                                get_sorted_specifiers(&import_specifiers_map),
                            ))),
                        )),
                        self.factory()
                            .new_string_literal(&import_source, TOKEN_FLAGS_NONE),
                        None,
                    );
                    set_parent_in_children(&s);
                    new_statements.push(s);
                }
                for e in new_statements {
                    statements =
                        self.insert_statement_after_custom_prologue(statements, Some(e));
                }
            } else if is_external_or_common_js_module(file) {
                statements_updated = true;
                let mut new_statements = Vec::new();
                for (import_source, import_specifiers_map) in
                    self.utilized_implicit_runtime_imports.clone()
                {
                    let sorted = get_sorted_specifiers(&import_specifiers_map);
                    let as_binding_elems: Vec<Arc<Node>> = sorted
                        .iter()
                        .map(|elem| {
                            self.factory().new_binding_element(
                                None,
                                elem.as_import_specifier().property_name.clone(),
                                Some(elem.as_import_specifier().name.clone()),
                                None,
                            )
                        })
                        .collect();
                    let binding_pattern = self.factory().new_binding_pattern(
                        SyntaxKind::ObjectBindingPattern,
                        &self.factory().new_node_list(as_binding_elems),
                    );
                    let require_call = self.factory().new_call_expression(
                        &self.factory().new_identifier("require"),
                        None,
                        None,
                        self.factory().new_node_list(vec![self
                            .factory()
                            .new_string_literal(&import_source, TOKEN_FLAGS_NONE)]),
                        NodeFlags::empty(),
                    );
                    let declaration = self.factory().new_variable_declaration(
                        &binding_pattern,
                        None,
                        None,
                        Some(&require_call),
                    );
                    let s = self.factory().new_variable_statement(
                        None,
                        &self.factory().new_variable_declaration_list(
                            &self.factory().new_node_list(vec![declaration]),
                            NodeFlags::Const,
                        ),
                    );
                    set_parent_in_children(&s);
                    new_statements.push(s);
                }
                for e in new_statements {
                    statements =
                        self.insert_statement_after_custom_prologue(statements, Some(e));
                }
            }
        }

        let visited = if statements_updated {
            self.factory()
                .update_source_file(&file.as_node(), self.factory().new_node_list(statements))
        } else {
            visited
        };

        self.current_source_file = None;
        self.import_specifier = String::new();
        self.filename_declaration = None;
        self.utilized_implicit_runtime_imports.clear();

        visited
    }

    pub fn visit_jsx_element(&mut self, element: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_jsx_element"); 
        let use_create_element = self.should_use_create_element(element);
        let start = skip_trivia(
            self.current_source_file.as_ref().unwrap().text(),
            element.pos(),
        );
        let location = TextRange::new(start, element.end());
        let jsx_el = element.as_jsx_element();
        let opening = jsx_el.opening_element.clone();
        let children = jsx_el.children.clone();
        if use_create_element {
            self.visit_jsx_opening_like_element_create_element(
                &opening,
                Some(&children),
                location,
            )
        } else {
            self.visit_jsx_opening_like_element_jsx(&opening, Some(&children), location)
        }
    }

    pub fn visit_jsx_self_closing_element(&mut self, element: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_jsx_self_closing_element"); 
        let use_create_element = self.should_use_create_element(element);
        let location = TextRange::new(
            skip_triva_of(self, element),
            element.end(),
        );
        if use_create_element {
            self.visit_jsx_opening_like_element_create_element(element, None, location)
        } else {
            self.visit_jsx_opening_like_element_jsx(element, None, location)
        }
    }

    pub fn visit_jsx_fragment(&mut self, fragment: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_jsx_fragment"); 
        let use_create_element = self.import_specifier.is_empty();
        let location = TextRange::new(
            skip_triva_of(self, fragment),
            fragment.end(),
        );
        let frag = fragment.as_jsx_fragment();
        let opening = frag.opening_fragment.clone();
        let children = frag.children.clone();
        if use_create_element {
            self.visit_jsx_opening_fragment_create_element(
                &opening,
                Some(&children),
                location,
            )
        } else {
            self.visit_jsx_opening_fragment_jsx(&opening, Some(&children), location)
        }
    }

    pub fn transform_jsx_child_to_expression(&mut self, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("transform_jsx_child_to_expression"); 
        let prev = self.in_jsx_child;
        self.set_in_child(true);
        let result = self.visitor().visit_node(Some(node));
        self.set_in_child(prev);
        result
    }

    fn convert_jsx_children_to_children_prop_assignment(
        &mut self,
        children: &[Arc<Node>],
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("convert_jsx_children_to_children_prop_assignment"); 
        let non_whitespace_children = get_semantic_jsx_children(children);
        if non_whitespace_children.len() == 1
            && (non_whitespace_children[0].kind != SyntaxKind::JsxExpression
                || non_whitespace_children[0]
                    .as_jsx_expression()
                    .dot_dot_dot_token
                    .is_none())
        {
            let result = self.transform_jsx_child_to_expression(&non_whitespace_children[0])?;
            return Some(self.factory().new_property_assignment(
                None,
                &self.factory().new_identifier("children"),
                None,
                None,
                &result,
            ));
        }
        let mut results = Vec::with_capacity(non_whitespace_children.len());
        for child in &non_whitespace_children {
            let Some(res) = self.transform_jsx_child_to_expression(child) else {
                continue;
            };
            let flags = self.emit_context().emit_flags(&res);
            self.emit_context()
                .set_emit_flags(&res, flags & !EmitFlags::START_ON_NEW_LINE);
            results.push(res);
        }
        if results.is_empty() {
            return None;
        }
        Some(self.factory().new_property_assignment(
            None,
            &self.factory().new_identifier("children"),
            None,
            None,
            &self
                .factory()
                .new_array_literal_expression(&self.factory().new_node_list(results), false),
        ))
    }

    fn convert_jsx_children_to_children_prop_object(
        &mut self,
        children: &[Arc<Node>],
    ) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("convert_jsx_children_to_children_prop_object"); 
        let prop = self.convert_jsx_children_to_children_prop_assignment(children)?;
        Some(self.factory().new_object_literal_expression(
            &self.factory().new_node_list(vec![prop]),
            false,
        ))
    }

    pub fn get_tag_name(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_tag_name"); 
        if node.kind == SyntaxKind::JsxElement {
            let opening = node.as_jsx_element().opening_element.clone();
            self.get_tag_name(&opening)
        } else if is_jsx_opening_like_element(node) {
            let tag_name = node.tag_name();
            if is_identifier(&tag_name) && is_intrinsic_jsx_name(tag_name.text()) {
                self.factory().new_string_literal(tag_name.text(), TOKEN_FLAGS_NONE)
            } else if is_jsx_namespaced_name(&tag_name) {
                let ns = tag_name.as_jsx_namespaced_name();
                self.factory().new_string_literal(
                    &format!("{}:{}", ns.namespace.text(), ns.name.text()),
                    TOKEN_FLAGS_NONE,
                )
            } else {
                create_expression_from_entity_name(&self.factory(), &tag_name)
            }
        } else {
            panic!("unhandled node kind passed to get_tag_name: {:?}", node.kind)
        }
    }

    pub fn visit_jsx_opening_like_element_jsx(
        &mut self,
        element: &Arc<Node>,
        children: Option<&NodeList>,
        location: TextRange,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_jsx_opening_like_element_jsx"); 
        let tag_name = self.get_tag_name(element);
        let mut children_prop = None;
        if let Some(children) = children
            && !children.nodes.is_empty()
        {
            children_prop = self.convert_jsx_children_to_children_prop_assignment(&children.nodes);
        }
        let mut key_attr: Option<Arc<Node>> = None;
        let mut attrs: Vec<Arc<Node>> = element.attributes_node().properties();
        for (i, p) in attrs.iter().enumerate() {
            if p.kind == SyntaxKind::JsxAttribute
                && is_identifier(&p.as_jsx_attribute().name)
                && p.as_jsx_attribute().name.text() == "key"
            {
                key_attr = Some(p.clone());
                attrs.remove(i);
                break;
            }
        }
        let object = if !attrs.is_empty() {
            self.transform_jsx_attributes_to_object_props(&attrs, children_prop)
        } else {
            let object_children = children_prop.into_iter().collect::<Vec<_>>();
            self.factory().new_object_literal_expression(
                &self.factory().new_node_list(object_children),
                false,
            )
        };
        self.visit_jsx_opening_like_element_or_fragment_jsx(
            tag_name,
            object,
            key_attr,
            children,
            location,
        )
    }

    pub fn transform_jsx_attributes_to_object_props(
        &mut self,
        attrs: &[Arc<Node>],
        children_prop: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_jsx_attributes_to_object_props"); 
        let target = self.compiler_options.get_emit_script_target();
        if target >= ScriptTarget::ES2018 {
            let props = self.transform_jsx_attributes_to_props(attrs, children_prop);
            return self.factory().new_object_literal_expression(
                &self.factory().new_node_list(props),
                false,
            );
        }
        self.transform_jsx_attributes_to_expression(attrs, children_prop)
    }

    pub fn transform_jsx_attributes_to_expression(
        &mut self,
        attrs: &[Arc<Node>],
        children_prop: Option<Arc<Node>>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_jsx_attributes_to_expression"); 
        let mut expressions: Vec<Arc<Node>> = Vec::with_capacity(2);
        let mut properties: Vec<Arc<Node>> = Vec::with_capacity(attrs.len());

        for attr in attrs {
            if is_jsx_spread_attribute(attr) {
                let clean_object = attr.expression().map_or(false, |e| {
                    is_object_literal_expression(e) && !has_proto(e)
                });
                if clean_object {
                    let expr = attr.expression().unwrap();
                    for prop in &expr.as_object_literal_expression().properties.nodes {
                        if is_spread_assignment(prop) {
                            let (e, p) =
                                self.combine_properties_into_new_expression(expressions, properties);
                            expressions = e;
                            properties = p;
                            expressions.push(
                                self.visitor().visit_node(prop.expression()).unwrap(),
                            );
                            continue;
                        }
                        properties.push(self.visitor().visit_node(Some(prop)).unwrap());
                    }
                    continue;
                }
                let (e, p) = self.combine_properties_into_new_expression(expressions, properties);
                expressions = e;
                properties = p;
                expressions.push(self.visitor().visit_node(attr.expression()).unwrap());
                continue;
            }
            properties.push(self.transform_jsx_attribute_to_object_literal_element(attr));
        }

        if let Some(children_prop) = children_prop {
            properties.push(children_prop);
        }

        let (mut expressions, _) = self.combine_properties_into_new_expression(expressions, properties);

        if !expressions.is_empty() && !is_object_literal_expression(&expressions[0]) {
            let empty_obj = self
                .factory()
                .new_object_literal_expression(&self.factory().new_node_list(vec![]), false);
            expressions.insert(0, empty_obj);
        }

        if expressions.len() == 1 {
            return expressions.pop().unwrap();
        }
        self.factory()
            .new_assign_helper(expressions, self.compiler_options.get_emit_script_target())
    }

    fn combine_properties_into_new_expression(
        &mut self,
        mut expressions: Vec<Arc<Node>>,
        props: Vec<Arc<Node>>,
    ) -> (Vec<Arc<Node>>, Vec<Arc<Node>>) { ::tsox_core::fntrace::enter("combine_properties_into_new_expression"); 
        if props.is_empty() {
            return (expressions, props);
        }
        let new_obj = self
            .factory()
            .new_object_literal_expression(&self.factory().new_node_list(props), false);
        expressions.push(new_obj);
        (expressions, Vec::new())
    }

    pub fn transform_jsx_attributes_to_props(
        &mut self,
        attrs: &[Arc<Node>],
        children_prop: Option<Arc<Node>>,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("transform_jsx_attributes_to_props"); 
        let mut props: Vec<Arc<Node>> = Vec::with_capacity(attrs.len());
        for attr in attrs {
            if attr.kind == SyntaxKind::JsxSpreadAttribute {
                let res = self.transform_jsx_spread_attributes_to_props(attr);
                props.extend(res);
            } else {
                props.push(self.transform_jsx_attribute_to_object_literal_element(attr));
            }
        }
        if let Some(children_prop) = children_prop {
            props.push(children_prop);
        }
        props
    }

    pub fn transform_jsx_spread_attributes_to_props(
        &mut self,
        node: &Arc<Node>,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("transform_jsx_spread_attributes_to_props"); 
        let clean_object = node.expression().map_or(false, |e| {
            is_object_literal_expression(e) && !has_proto(e)
        });
        if clean_object {
            let expr = node.expression().unwrap();
            let (res, _) = self
                .visitor()
                .visit_slice(&expr.as_object_literal_expression().properties.nodes);
            return res;
        }
        let visited =
            self.visitor().visit_node(node.expression()).unwrap();
        vec![self.factory().new_spread_assignment(visited)]
    }

    pub fn transform_jsx_attribute_to_object_literal_element(
        &mut self,
        node: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_jsx_attribute_to_object_literal_element"); 
        let name = self.get_attribute_name(node);
        let expression = self
            .transform_jsx_attribute_initializer(node.as_jsx_attribute().initializer.as_ref());
        self.factory()
            .new_property_assignment(None, &name, None, None, &expression)
    }

    pub fn get_attribute_name(&mut self, node: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_attribute_name"); 
        let name = &node.as_jsx_attribute().name;
        if is_identifier(name) {
            let text = name.text();
            if is_identifier_text(text, LanguageVariant::Standard) {
                return name.clone();
            }
            return self.factory().new_string_literal(text, TOKEN_FLAGS_NONE);
        }
        let ns = name.as_jsx_namespaced_name();
        self.factory().new_string_literal(
            &format!("{}:{}", ns.namespace.text(), ns.name.text()),
            TOKEN_FLAGS_NONE,
        )
    }

    pub fn transform_jsx_attribute_initializer(&mut self, node: Option<&Arc<Node>>) -> Arc<Node> { ::tsox_core::fntrace::enter("transform_jsx_attribute_initializer"); 
        let Some(node) = node else {
            return self.factory().new_true_expression();
        };
        if node.kind == SyntaxKind::StringLiteral {
            let mut res = self.factory().new_string_literal(
                &decode_entities(node.text()),
                node.as_string_literal().token_flags,
            );
            if let Some(n) = Arc::get_mut(&mut res) {
                n.loc = node.loc;
            }
            return res;
        }
        if node.kind == SyntaxKind::JsxExpression {
            if node.expression().is_none() {
                return self.factory().new_true_expression();
            }
            return self.visitor().visit_node(node.expression()).unwrap();
        }
        if is_jsx_element(node) || is_jsx_self_closing_element(node) || is_jsx_fragment(node) {
            self.set_in_child(false);
            return self.visitor().visit_node(Some(node)).unwrap();
        }
        panic!("Unhandled node kind found in jsx initializer: {:?}", node.kind)
    }

    pub fn visit_jsx_opening_like_element_or_fragment_jsx(
        &mut self,
        tag_name: Arc<Node>,
        object: Arc<Node>,
        key_attr: Option<Arc<Node>>,
        children: Option<&NodeList>,
        location: TextRange,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_jsx_opening_like_element_or_fragment_jsx"); 
        let mut non_whitespace_children: Vec<Arc<Node>> = Vec::new();
        if let Some(children) = children {
            non_whitespace_children = get_semantic_jsx_children(&children.nodes);
        }
        let is_static_children = non_whitespace_children.len() > 1
            || (non_whitespace_children.len() == 1
                && is_jsx_expression(&non_whitespace_children[0])
                && non_whitespace_children[0]
                    .as_jsx_expression()
                    .dot_dot_dot_token
                    .is_some());
        let mut args: Vec<Arc<Node>> = Vec::with_capacity(3);
        args.push(tag_name);
        args.push(object);
        if let Some(key_attr) = &key_attr {
            let key_initializer = key_attr.as_jsx_attribute().initializer.as_ref();
            args.push(self.transform_jsx_attribute_initializer(key_initializer));
        }

        if self.compiler_options.jsx == JsxEmit::ReactJSXDev {
            let current_file = self.current_source_file.clone();
            let file_node = current_file.as_ref().unwrap().as_node().clone();
            let original_file = self.emit_context().most_original(&file_node);
            if is_source_file(&original_file) {
                if key_attr.is_none() {
                    args.push(self.factory().new_void_zero_expression());
                }
                if is_static_children {
                    args.push(self.factory().new_true_expression());
                } else {
                    args.push(self.factory().new_false_expression());
                }
                let (line, col) = get_ecma_line_and_utf16_character_of_position(
                    self.current_source_file.as_ref().unwrap(),
                    location.pos(),
                );
                let file_name_expression = self.get_current_file_name_expression();
                args.push(self.factory().new_property_assignment(
                    None,
                    &self.factory().new_identifier("fileName"),
                    None,
                    None,
                    &file_name_expression,
                ));
                let line_assignment = self.factory().new_property_assignment(
                    None,
                    &self.factory().new_identifier("lineNumber"),
                    None,
                    None,
                    &self.factory().new_numeric_literal(
                        &format!("{}", line + 1),
                        TOKEN_FLAGS_NONE,
                    ),
                );
                args.push(line_assignment);
                let column_assignment = self.factory().new_property_assignment(
                    None,
                    &self.factory().new_identifier("columnNumber"),
                    None,
                    None,
                    &self.factory().new_numeric_literal(
                        &format!("{}", col + 1),
                        TOKEN_FLAGS_NONE,
                    ),
                );
                args.push(column_assignment);
                let assignments = args.split_off(args.len() - 3);
                let dev_metadata = self.factory().new_object_literal_expression(
                    &self.factory().new_node_list(assignments),
                    false,
                );
                args.push(dev_metadata);
                args.push(self.factory().new_this_expression());
            }
        }

        let callee = self.get_jsx_factory_callee(is_static_children);
        let mut element = self.factory().new_call_expression(
            &callee,
            None,
            None,
            self.factory().new_node_list(args),
            NodeFlags::empty(),
        );
        if let Some(n) = Arc::get_mut(&mut element) {
            n.loc = location;
        }

        if self.in_jsx_child {
            self.emit_context()
                .add_emit_flags(&element, EmitFlags::START_ON_NEW_LINE);
        }

        element
    }

    pub fn visit_jsx_opening_fragment_jsx(
        &mut self,
        fragment: &Arc<Node>,
        children: Option<&NodeList>,
        location: TextRange,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_jsx_opening_fragment_jsx"); 
        let mut children_props: Option<Arc<Node>> = None;
        if let Some(children) = children
            && !children.nodes.is_empty()
        {
            children_props = self.convert_jsx_children_to_children_prop_object(&children.nodes);
        }
        let children_props = children_props.unwrap_or_else(|| {
            self.factory()
                .new_object_literal_expression(&self.factory().new_node_list(vec![]), false)
        });
        let fragment_reference = self.get_implicit_jsx_fragment_reference();
        self.visit_jsx_opening_like_element_or_fragment_jsx(
            fragment_reference,
            children_props,
            None,
            children,
            location,
        )
    }

    pub fn create_react_namespace(
        &mut self,
        react_namespace: &str,
        parent: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_react_namespace"); 
        let react_namespace = if react_namespace.is_empty() {
            "React"
        } else {
            react_namespace
        };
        let mut react = self.factory().new_identifier(react_namespace);
        if let Some(n) = Arc::get_mut(&mut react) {
            n.flags.remove(NodeFlags::Synthesized);
        }

        if let Some(parsed_parent) = self.emit_context().parse_node(parent) {
            react.set_parent(&parsed_parent);
        }

        if let Some(container) =
            self.emit_resolver
                .get_referenced_export_container_r40k08(&react, false)
            && is_module_declaration(&container)
        {
            let generated = self.factory().new_generated_name_for_node(&container);
            let container_name = self.factory().generated_name_node(&generated);
            return self.factory().new_property_access_expression(
                &container_name,
                None,
                &react,
                NodeFlags::empty(),
            );
        }

        react
    }

    pub fn create_jsx_factory_expression_from_entity_name(
        &mut self,
        e: &Arc<Node>,
        parent: &Arc<Node>,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_jsx_factory_expression_from_entity_name"); 
        if is_qualified_name(e) {
            let left_data = e.as_qualified_name();
            let left = self.create_jsx_factory_expression_from_entity_name(&left_data.left, parent);
            let right = self.factory().new_identifier(left_data.right.text());
            return self
                .factory()
                .new_property_access_expression(&left, None, &right, NodeFlags::empty());
        }
        self.create_react_namespace(e.text(), parent)
    }

    pub fn create_jsx_pseudo_factory_expression(
        &mut self,
        parent: &Arc<Node>,
        e: Option<&Arc<Node>>,
        target: &str,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("create_jsx_pseudo_factory_expression"); 
        if let Some(e) = e {
            return self.create_jsx_factory_expression_from_entity_name(e, parent);
        }
        let react_namespace = self.create_react_namespace(&self.compiler_options.react_namespace.clone(), parent);
        let target_identifier = self.factory().new_identifier(target);
        self.factory()
            .new_property_access_expression(&react_namespace, None, &target_identifier, NodeFlags::empty())
    }

    pub fn create_jsx_factory_expression(&mut self, parent: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("create_jsx_factory_expression"); 
        let current_file = self.current_source_file.clone();
        let file_node = current_file.as_ref().unwrap().as_node().clone();
        let e = self
            .emit_resolver
            .get_jsx_factory_entity_r40k08(&file_node);
        self.create_jsx_pseudo_factory_expression(parent, e.as_ref(), "createElement")
    }

    pub fn create_jsx_fragment_factory_expression(&mut self, parent: &Arc<Node>) -> Arc<Node> { ::tsox_core::fntrace::enter("create_jsx_fragment_factory_expression"); 
        let current_file = self.current_source_file.clone();
        let file_node = current_file.as_ref().unwrap().as_node().clone();
        let e = self
            .emit_resolver
            .get_jsx_fragment_factory_entity_r40k08(&file_node);
        self.create_jsx_pseudo_factory_expression(parent, e.as_ref(), "Fragment")
    }

    pub fn visit_jsx_opening_like_element_create_element(
        &mut self,
        element: &Arc<Node>,
        children: Option<&NodeList>,
        location: TextRange,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_jsx_opening_like_element_create_element"); 
        let tag_name = self.get_tag_name(element);
        let attrs = element.attributes_node().properties();
        let object_properties = if !attrs.is_empty() {
            self.transform_jsx_attributes_to_object_props(&attrs, None)
        } else {
            self.factory().new_keyword_expression(SyntaxKind::NullKeyword)
        };

        let callee = if self.import_specifier.is_empty() {
            self.create_jsx_factory_expression(element)
        } else {
            self.get_implicit_import_for_name("createElement")
        };

        let new_children = self.transform_jsx_children(children);

        if new_children.len() > 1 {
            for child in &new_children {
                self.emit_context()
                    .add_emit_flags(child, EmitFlags::START_ON_NEW_LINE);
            }
        }

        let mut args: Vec<Arc<Node>> = Vec::with_capacity(new_children.len() + 2);
        args.push(tag_name);
        args.push(object_properties);
        args.extend(new_children);

        let mut result = self.factory().new_call_expression(
            &callee,
            None,
            None,
            self.factory().new_node_list(args),
            NodeFlags::empty(),
        );
        if let Some(n) = Arc::get_mut(&mut result) {
            n.loc = location;
        }

        if self.in_jsx_child {
            self.emit_context()
                .add_emit_flags(&result, EmitFlags::START_ON_NEW_LINE);
        }
        result
    }

    pub fn visit_jsx_opening_fragment_create_element(
        &mut self,
        fragment: &Arc<Node>,
        children: Option<&NodeList>,
        location: TextRange,
    ) -> Arc<Node> { ::tsox_core::fntrace::enter("visit_jsx_opening_fragment_create_element"); 
        let tag_name = self.create_jsx_fragment_factory_expression(fragment);
        let callee = self.create_jsx_factory_expression(fragment);

        let new_children = self.transform_jsx_children(children);

        if new_children.len() > 1 {
            for child in &new_children {
                self.emit_context()
                    .add_emit_flags(child, EmitFlags::START_ON_NEW_LINE);
            }
        }

        let mut args: Vec<Arc<Node>> = Vec::with_capacity(new_children.len() + 2);
        args.push(tag_name);
        args.push(self.factory().new_keyword_expression(SyntaxKind::NullKeyword));
        args.extend(new_children);

        let mut result = self.factory().new_call_expression(
            &callee,
            None,
            None,
            self.factory().new_node_list(args),
            NodeFlags::empty(),
        );
        if let Some(n) = Arc::get_mut(&mut result) {
            n.loc = location;
        }

        if self.in_jsx_child {
            self.emit_context()
                .add_emit_flags(&result, EmitFlags::START_ON_NEW_LINE);
        }
        result
    }

    fn transform_jsx_children(&mut self, children: Option<&NodeList>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("transform_jsx_children"); 
        let mut new_children = Vec::new();
        if let Some(children) = children {
            for c in &children.nodes {
                if let Some(res) = self.transform_jsx_child_to_expression(c) {
                    new_children.push(res);
                }
            }
        }
        new_children
    }

    pub fn visit_jsx_text(&mut self, text: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_jsx_text"); 
        let fixed = fixup_whitespace_and_decode_entities(text.text());
        if fixed.is_empty() {
            return None;
        }
        Some(self.factory().new_string_literal(&fixed, TOKEN_FLAGS_NONE))
    }

    pub fn visit_jsx_expression(&mut self, expression: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("visit_jsx_expression"); 
        let data = expression.as_jsx_expression();
        let e = self.visitor().visit_node(data.expression.as_ref());
        if data.dot_dot_dot_token.is_some() {
            return Some(self.factory().new_spread_element(e?));
        }
        e
    }

    pub fn should_use_create_element(&self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_use_create_element"); 
        self.import_specifier.is_empty() || has_key_after_props_spread(node)
    }

    pub fn is_any_prologue_directive(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_any_prologue_directive"); 
        is_prologue_directive(node)
            || self
                .emit_context()
                .emit_flags(node)
                .intersects(EmitFlags::CUSTOM_PROLOGUE)
    }

    pub fn insert_statement_after_custom_prologue(
        &mut self,
        to: Vec<Arc<Node>>,
        statement: Option<Arc<Node>>,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("insert_statement_after_custom_prologue"); 
        insert_statement_after_prologue(to, statement, |tx, node| {
            tx.is_any_prologue_directive(node)
        }, self)
    }
}

fn skip_triva_of(tx: &JsxTransformer, node: &Arc<Node>) -> usize { ::tsox_core::fntrace::enter("skip_triva_of"); 
    skip_trivia(tx.current_source_file.as_ref().unwrap().text(), node.pos())
}

pub fn has_key_after_props_spread(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_key_after_props_spread"); 
    let mut spread = false;
    let opener = if node.kind == SyntaxKind::JsxElement {
        node.opening_element()
    } else {
        node.clone()
    };
    for elem in opener.attributes_node().properties() {
        let blocks_spread = elem.expression().map_or(false, |e| {
            is_object_literal_expression(e)
                && !has_proto(e)
                && !e.as_object_literal_expression()
                    .properties
                    .nodes
                    .iter()
                    .any(|p| is_spread_assignment(p))
        });
        if is_jsx_spread_attribute(&elem) && !blocks_spread {
            spread = true;
        } else if spread
            && is_jsx_attribute(&elem)
            && elem
                .name()
                .map_or(false, |n| is_identifier(n) && n.text() == "key")
        {
            return true;
        }
    }
    false
}

pub fn insert_statement_after_prologue(
    mut to: Vec<Arc<Node>>,
    statement: Option<Arc<Node>>,
    is_prologue_directive: impl Fn(&mut JsxTransformer, &Arc<Node>) -> bool,
    tx: &mut JsxTransformer,
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("insert_statement_after_prologue"); 
    let Some(statement) = statement else {
        return to;
    };
    let mut statement_idx = 0;
    while statement_idx < to.len() {
        if !is_prologue_directive(tx, &to[statement_idx]) {
            break;
        }
        statement_idx += 1;
    }
    to.insert(statement_idx, statement);
    to
}

pub fn sort_import_specifiers(a: &Arc<Node>, b: &Arc<Node>) -> std::cmp::Ordering { ::tsox_core::fntrace::enter("sort_import_specifiers"); 
    let a_data = a.as_import_specifier();
    let b_data = b.as_import_specifier();
    let a_property = a_data.property_name.as_ref().unwrap_or(&a_data.name);
    let b_property = b_data.property_name.as_ref().unwrap_or(&b_data.name);
    let res = compare_strings_case_sensitive(a_property.text(), b_property.text());
    if res != 0 {
        return res.cmp(&0);
    }
    compare_strings_case_sensitive(
        a_data.name.text(),
        b_data.name.text(),
    )
    .cmp(&0)
}

pub fn get_sorted_specifiers(m: &HashMap<String, Arc<Node>>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_sorted_specifiers"); 
    let mut res: Vec<Arc<Node>> = m.values().cloned().collect();
    res.sort_by(sort_import_specifiers);
    res
}

pub fn has_proto(obj: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_proto"); 
    obj.as_object_literal_expression()
        .properties
        .nodes
        .iter()
        .any(|p| {
            is_property_assignment(p)
                && p.name().map_or(false, |n| {
                    (is_string_literal(n) || is_identifier(n)) && n.text() == "__proto__"
                })
        })
}

fn add_line_of_jsx_text(acc: &mut String, trimmed_line: &str, is_initial: bool) { ::tsox_core::fntrace::enter("add_line_of_jsx_text"); 
    let decoded = decode_entities(trimmed_line);
    if !is_initial {
        acc.push(' ');
    }
    acc.push_str(&decoded);
}

pub fn fixup_whitespace_and_decode_entities(text: &str) -> String { ::tsox_core::fntrace::enter("fixup_whitespace_and_decode_entities"); 
    let mut acc = String::new();
    let mut initial = true;
    let mut first_non_whitespace: i64 = 0;
    let mut last_non_whitespace_end: i64 = -1;
    let mut iter = text.char_indices().peekable();
    while let Some((i, c)) = iter.next() {
        let size = c.len_utf8();
        if is_line_break(c) {
            if first_non_whitespace != -1 && last_non_whitespace_end != -1 {
                add_line_of_jsx_text(
                    &mut acc,
                    &text[first_non_whitespace as usize..(last_non_whitespace_end + 1) as usize],
                    initial,
                );
                initial = false;
            }
            first_non_whitespace = -1;
        } else if !is_white_space_single_line(c) {
            last_non_whitespace_end = (i + size) as i64 - 1;
            if first_non_whitespace == -1 {
                first_non_whitespace = i as i64;
            }
        }
    }

    if first_non_whitespace != -1 {
        add_line_of_jsx_text(&mut acc, &text[first_non_whitespace as usize..], initial);
    }
    acc
}

pub fn decode_entities(text: &str) -> String { ::tsox_core::fntrace::enter("decode_entities"); 
    let Some(mut i) = text.find('&') else {
        return text.to_string();
    };

    let mut result = String::with_capacity(text.len());
    let mut text = text;
    loop {
        result.push_str(&text[..i]);
        text = &text[i..];

        let Some(mut semi) = text.find(';') else {
            break;
        };

        loop {
            let Some(next_amp) = text[1..semi].find('&') else {
                break;
            };
            result.push_str(&text[..next_amp + 1]);
            text = &text[next_amp + 1..];
            semi -= next_amp + 1;
        }

        let entity = &text[1..semi];
        if let Some(decoded) = decode_entity(entity) {
            result.push_str(&encode_js_string_rune(decoded as u32));
        } else {
            result.push_str(&text[..semi + 1]);
        }
        text = &text[semi + 1..];

        let Some(next) = text.find('&') else {
            break;
        };
        i = next;
    }
    result.push_str(text);
    result
}

pub fn decode_entity(entity: &str) -> Option<char> { ::tsox_core::fntrace::enter("decode_entity"); 
    if entity.is_empty() {
        return None;
    }

    if let Some(entity) = entity.strip_prefix('#') {
        if entity.is_empty() {
            return None;
        }

        let mut base = 10;
        let entity = if let Some(entity) = entity.strip_prefix('x') {
            base = 16;
            entity
        } else {
            entity
        };

        if entity.is_empty() {
            return None;
        }

        for c in entity.chars() {
            if base == 16 && !c.is_ascii_hexdigit() {
                return None;
            }
            if base == 10 && !c.is_ascii_digit() {
                return None;
            }
        }

        let parsed = i64::from_str_radix(entity, base).ok()?;
        return char::from_u32(parsed as u32);
    }

    entity_lookup(entity)
}

fn entity_lookup(entity: &str) -> Option<char> { ::tsox_core::fntrace::enter("entity_lookup"); 
    const ENTITIES: &[(&str, u32)] = &[
        ("quot", 0x0022), ("amp", 0x0026), ("apos", 0x0027), ("lt", 0x003C), ("gt", 0x003E),
        ("nbsp", 0x00A0), ("iexcl", 0x00A1), ("cent", 0x00A2), ("pound", 0x00A3), ("curren", 0x00A4),
        ("yen", 0x00A5), ("brvbar", 0x00A6), ("sect", 0x00A7), ("uml", 0x00A8), ("copy", 0x00A9),
        ("ordf", 0x00AA), ("laquo", 0x00AB), ("not", 0x00AC), ("shy", 0x00AD), ("reg", 0x00AE),
        ("macr", 0x00AF), ("deg", 0x00B0), ("plusmn", 0x00B1), ("sup2", 0x00B2), ("sup3", 0x00B3),
        ("acute", 0x00B4), ("micro", 0x00B5), ("para", 0x00B6), ("middot", 0x00B7), ("cedil", 0x00B8),
        ("sup1", 0x00B9), ("ordm", 0x00BA), ("raquo", 0x00BB), ("frac14", 0x00BC), ("frac12", 0x00BD),
        ("frac34", 0x00BE), ("iquest", 0x00BF), ("Agrave", 0x00C0), ("Aacute", 0x00C1), ("Acirc", 0x00C2),
        ("Atilde", 0x00C3), ("Auml", 0x00C4), ("Aring", 0x00C5), ("AElig", 0x00C6), ("Ccedil", 0x00C7),
        ("Egrave", 0x00C8), ("Eacute", 0x00C9), ("Ecirc", 0x00CA), ("Euml", 0x00CB), ("Igrave", 0x00CC),
        ("Iacute", 0x00CD), ("Icirc", 0x00CE), ("Iuml", 0x00CF), ("ETH", 0x00D0), ("Ntilde", 0x00D1),
        ("Ograve", 0x00D2), ("Oacute", 0x00D3), ("Ocirc", 0x00D4), ("Otilde", 0x00D5), ("Ouml", 0x00D6),
        ("times", 0x00D7), ("Oslash", 0x00D8), ("Ugrave", 0x00D9), ("Uacute", 0x00DA), ("Ucirc", 0x00DB),
        ("Uuml", 0x00DC), ("Yacute", 0x00DD), ("THORN", 0x00DE), ("szlig", 0x00DF), ("agrave", 0x00E0),
        ("aacute", 0x00E1), ("acirc", 0x00E2), ("atilde", 0x00E3), ("auml", 0x00E4), ("aring", 0x00E5),
        ("aelig", 0x00E6), ("ccedil", 0x00E7), ("egrave", 0x00E8), ("eacute", 0x00E9), ("ecirc", 0x00EA),
        ("euml", 0x00EB), ("igrave", 0x00EC), ("iacute", 0x00ED), ("icirc", 0x00EE), ("iuml", 0x00EF),
        ("eth", 0x00F0), ("ntilde", 0x00F1), ("ograve", 0x00F2), ("oacute", 0x00F3), ("ocirc", 0x00F4),
        ("otilde", 0x00F5), ("ouml", 0x00F6), ("divide", 0x00F7), ("oslash", 0x00F8), ("ugrave", 0x00F9),
        ("uacute", 0x00FA), ("ucirc", 0x00FB), ("uuml", 0x00FC), ("yacute", 0x00FD), ("thorn", 0x00FE),
        ("yuml", 0x00FF), ("OElig", 0x0152), ("oelig", 0x0153), ("Scaron", 0x0160), ("scaron", 0x0161),
        ("Yuml", 0x0178), ("fnof", 0x0192), ("circ", 0x02C6), ("tilde", 0x02DC), ("Alpha", 0x0391),
        ("Beta", 0x0392), ("Gamma", 0x0393), ("Delta", 0x0394), ("Epsilon", 0x0395), ("Zeta", 0x0396),
        ("Eta", 0x0397), ("Theta", 0x0398), ("Iota", 0x0399), ("Kappa", 0x039A), ("Lambda", 0x039B),
        ("Mu", 0x039C), ("Nu", 0x039D), ("Xi", 0x039E), ("Omicron", 0x039F), ("Pi", 0x03A0),
        ("Rho", 0x03A1), ("Sigma", 0x03A3), ("Tau", 0x03A4), ("Upsilon", 0x03A5), ("Phi", 0x03A6),
        ("Chi", 0x03A7), ("Psi", 0x03A8), ("Omega", 0x03A9), ("alpha", 0x03B1), ("beta", 0x03B2),
        ("gamma", 0x03B3), ("delta", 0x03B4), ("epsilon", 0x03B5), ("zeta", 0x03B6), ("eta", 0x03B7),
        ("theta", 0x03B8), ("iota", 0x03B9), ("kappa", 0x03BA), ("lambda", 0x03BB), ("mu", 0x03BC),
        ("nu", 0x03BD), ("xi", 0x03BE), ("omicron", 0x03BF), ("pi", 0x03C0), ("rho", 0x03C1),
        ("sigmaf", 0x03C2), ("sigma", 0x03C3), ("tau", 0x03C4), ("upsilon", 0x03C5), ("phi", 0x03C6),
        ("chi", 0x03C7), ("psi", 0x03C8), ("omega", 0x03C9), ("thetasym", 0x03D1), ("upsih", 0x03D2),
        ("piv", 0x03D6), ("ensp", 0x2002), ("emsp", 0x2003), ("thinsp", 0x2009), ("zwnj", 0x200C),
        ("zwj", 0x200D), ("lrm", 0x200E), ("rlm", 0x200F), ("ndash", 0x2013), ("mdash", 0x2014),
        ("lsquo", 0x2018), ("rsquo", 0x2019), ("sbquo", 0x201A), ("ldquo", 0x201C), ("rdquo", 0x201D),
        ("bdquo", 0x201E), ("dagger", 0x2020), ("Dagger", 0x2021), ("bull", 0x2022), ("hellip", 0x2026),
        ("permil", 0x2030), ("prime", 0x2032), ("Prime", 0x2033), ("lsaquo", 0x2039), ("rsaquo", 0x203A),
        ("oline", 0x203E), ("frasl", 0x2044), ("euro", 0x20AC), ("image", 0x2111), ("weierp", 0x2118),
        ("real", 0x211C), ("trade", 0x2122), ("alefsym", 0x2135), ("larr", 0x2190), ("uarr", 0x2191),
        ("rarr", 0x2192), ("darr", 0x2193), ("harr", 0x2194), ("crarr", 0x21B5), ("lArr", 0x21D0),
        ("uArr", 0x21D2), ("rArr", 0x21D3), ("hArr", 0x21D4), ("forall", 0x2200),
        ("part", 0x2202), ("exist", 0x2203), ("empty", 0x2205), ("nabla", 0x2207), ("isin", 0x2208),
        ("notin", 0x2209), ("ni", 0x220B), ("prod", 0x220F), ("sum", 0x2211), ("minus", 0x2212),
        ("lowast", 0x2217), ("radic", 0x221A), ("prop", 0x221D), ("infin", 0x221E), ("ang", 0x2220),
        ("and", 0x2227), ("or", 0x2228), ("cap", 0x2227), ("cup", 0x222A), ("int", 0x222B),
        ("there4", 0x2234), ("sim", 0x223C), ("cong", 0x2245), ("asymp", 0x2248), ("ne", 0x2260),
        ("equiv", 0x2261), ("le", 0x2264), ("ge", 0x2265), ("sub", 0x2282), ("sup", 0x2283),
        ("nsub", 0x2284), ("sube", 0x2286), ("supe", 0x2287), ("oplus", 0x2295), ("otimes", 0x2297),
        ("perp", 0x22A5), ("sdot", 0x22C5), ("lceil", 0x2308), ("rceil", 0x2309), ("lfloor", 0x230A),
        ("rfloor", 0x230B), ("lang", 0x2329), ("rang", 0x232A), ("loz", 0x25CA), ("spades", 0x2660),
        ("clubs", 0x2663), ("hearts", 0x2665), ("diams", 0x2666),
    ];
    ENTITIES
        .iter()
        .find(|(name, _)| *name == entity)
        .and_then(|(_, cp)| char::from_u32(*cp))
}
