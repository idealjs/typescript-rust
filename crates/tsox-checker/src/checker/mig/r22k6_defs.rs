use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::node_data_generated::{
    ClassDeclarationData, ConstructorDeclarationData, ExportDeclarationData, ExportSpecifierData,
    ExpressionStatementData, HeritageClauseData, InterfaceDeclarationData, MethodDeclarationData,
    ModuleBlockData, ModuleDeclarationData, NamedExportsData, NodeData, PropertyDeclarationData,
    PropertySignatureDeclarationData, TypeAliasDeclarationData, TypeQueryNodeData,
    TypeReferenceNodeData, VariableDeclarationData, VariableDeclarationListData,
    VariableStatementData,
};
use tsox_frontend::ast::{ModifierList, Node, NodeList, SyntaxKind};

use crate::checker::checker_checker_checker::Checker;
use crate::checker::mig::m2a::r20k6_defs::PatternAmbientModule;
use crate::checker::nodecopy_builder::{EmitContextStub, NodeBuilderImpl, NodeFactoryStub};
use crate::checker::types::{ObjectFlags, Signature, Type, TypeFlags, TypeFacts};
use crate::checker::types_cached_type_kind::CacheHashKey;

thread_local! {
    pub static PATTERN_AMBIENT_MODULES: RefCell<Vec<PatternAmbientModule>> =
        const { RefCell::new(Vec::new()) };
}

pub fn take_pattern_ambient_modules() -> Vec<PatternAmbientModule> {
    PATTERN_AMBIENT_MODULES.with(|m| std::mem::take(&mut *m.borrow_mut()))
}

pub fn set_pattern_ambient_modules(modules: Vec<PatternAmbientModule>) {
    PATTERN_AMBIENT_MODULES.with(|m| *m.borrow_mut() = modules);
}

thread_local! {
    pub static SUBTYPE_REDUCTION_CACHE: RefCell<HashMap<CacheHashKey, Vec<Arc<Type>>>> =
        RefCell::new(HashMap::new());
}

pub fn subtype_reduction_cache_get(key: &CacheHashKey) -> Option<Vec<Arc<Type>>> {
    SUBTYPE_REDUCTION_CACHE.with(|c| c.borrow().get(key).cloned())
}

pub fn subtype_reduction_cache_insert(key: CacheHashKey, types: Vec<Arc<Type>>) {
    SUBTYPE_REDUCTION_CACHE.with(|c| c.borrow_mut().insert(key, types));
}

impl Checker {
    pub fn get_type_facts(&mut self, t: &Arc<Type>, mask: TypeFacts) -> TypeFacts {
        self.get_type_facts_worker(t, mask) & mask
    }
}

pub fn node_modifier_flags(node: &Arc<Node>) -> tsox_frontend::ast::ModifierFlags {
    node.modifiers()
        .map(|m| m.modifier_flags)
        .unwrap_or_default()
}

pub fn set_emit_flags_single_line(_e: &EmitContextStub, _node: &Arc<Node>) {}

pub struct SignatureToSignatureDeclarationOptions {
    pub name: Arc<Node>,
}

fn token_node(kind: SyntaxKind) -> Arc<Node> {
    Arc::new(Node::new(kind, NodeData::Token))
}

pub trait R22K6NodeFactoryExt {
    fn new_property_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        question_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node>;
    fn new_method_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        asterisk_token: Option<Arc<Node>>,
        name: Arc<Node>,
        question_token: Option<Arc<Node>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: Arc<NodeList>,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        body: Option<Arc<Node>>,
    ) -> Arc<Node>;
    fn new_class_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        type_parameters: NodeList,
        heritage_clauses: NodeList,
        members: NodeList,
    ) -> Arc<Node>;
    fn new_interface_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        type_parameters: NodeList,
        heritage_clauses: NodeList,
        members: NodeList,
    ) -> Arc<Node>;
    fn new_heritage_clause(&self, token: SyntaxKind, types: NodeList) -> Arc<Node>;
    fn new_property_signature_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        question_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node>;
    fn new_constructor_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: NodeList,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        body: Option<Arc<Node>>,
    ) -> Arc<Node>;
    fn new_variable_statement(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        declaration_list: Arc<Node>,
    ) -> Arc<Node>;
    fn new_variable_declaration_list(
        &self,
        declarations: NodeList,
        _flags: tsox_frontend::ast::NodeFlags,
    ) -> Arc<Node>;
    fn new_variable_declaration(
        &self,
        name: Arc<Node>,
        exclamation_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node>;
    fn new_expression_statement(&self, expression: Arc<Node>) -> Arc<Node>;
    fn new_export_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        is_type_only: bool,
        export_clause: Arc<Node>,
        module_specifier: Option<Arc<Node>>,
        attributes: Option<Arc<Node>>,
    ) -> Arc<Node>;
    fn new_named_exports(&self, elements: NodeList) -> Arc<Node>;
    fn new_export_specifier(
        &self,
        is_type_only: bool,
        property_name: Option<Arc<Node>>,
        name: Arc<Node>,
    ) -> Arc<Node>;
    fn new_module_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        keyword: SyntaxKind,
        name: Arc<Node>,
        attributes: Option<Arc<Node>>,
        body: Arc<Node>,
    ) -> Arc<Node>;
    fn new_module_block(&self, statements: NodeList) -> Arc<Node>;
    fn new_type_alias_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        type_parameters: NodeList,
        type_node: Option<Arc<Node>>,
    ) -> Arc<Node>;
    fn new_type_reference_node(
        &self,
        type_name: &Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node>;
    fn new_property_access_expression(
        &self,
        expression: &Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        name: &Arc<Node>,
        _flags: tsox_frontend::ast::NodeFlags,
    ) -> Arc<Node>;
    fn new_indexed_access_type_node(
        &self,
        object_type: &Arc<Node>,
        index_type: &Arc<Node>,
    ) -> Arc<Node>;
    fn new_parenthesized_type_node(&self, type_node: &Arc<Node>) -> Arc<Node>;
    fn new_type_query_node(&self, expr_name: &Arc<Node>) -> Arc<Node>;
    fn new_literal_type_node(&self, literal: Arc<Node>) -> Arc<Node>;
    fn new_element_access_expression(
        &self,
        expression: &Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        argument_expression: &Arc<Node>,
    ) -> Arc<Node>;
    fn update_type_reference_node(
        &self,
        node: &Arc<Node>,
        type_name: &Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node>;
    fn update_import_type_node(
        &self,
        node: &Arc<Node>,
        _argument: Option<&Arc<Node>>,
        _qualifier: Option<&Arc<Node>>,
        _type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node>;
    fn add_synthetic_leading_comment(
        &self,
        node: &Arc<Node>,
        _kind: SyntaxKind,
        _text: &str,
    ) -> Arc<Node>;
    fn new_keyword_type_node_ex(&self, kind: SyntaxKind) -> Arc<Node>;
    fn update_class_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: Option<Arc<Node>>,
        type_parameters: Option<Arc<NodeList>>,
        heritage_clauses: Option<Arc<NodeList>>,
        members: Arc<NodeList>,
    ) -> Arc<Node>;
}

impl R22K6NodeFactoryExt for NodeFactoryStub {
    fn new_property_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        question_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::PropertyDeclaration,
            NodeData::PropertyDeclaration(PropertyDeclarationData {
                modifiers,
                name,
                postfix_token: question_token,
                type_node,
                initializer,
            }),
        ))
    }

    fn new_method_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        asterisk_token: Option<Arc<Node>>,
        name: Arc<Node>,
        question_token: Option<Arc<Node>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: Arc<NodeList>,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        body: Option<Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::MethodDeclaration,
            NodeData::MethodDeclaration(MethodDeclarationData {
                modifiers,
                asterisk_token,
                name,
                postfix_token: question_token,
                type_parameters,
                parameters,
                type_node,
                full_signature,
                body,
            }),
        ))
    }

    fn new_class_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        type_parameters: NodeList,
        heritage_clauses: NodeList,
        members: NodeList,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ClassDeclaration,
            NodeData::ClassDeclaration(ClassDeclarationData {
                modifiers,
                name: Some(name),
                type_parameters: Some(Arc::new(type_parameters)),
                heritage_clauses: Some(Arc::new(heritage_clauses)),
                members: Arc::new(members),
            }),
        ))
    }

    fn new_interface_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        type_parameters: NodeList,
        heritage_clauses: NodeList,
        members: NodeList,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::InterfaceDeclaration,
            NodeData::InterfaceDeclaration(InterfaceDeclarationData {
                modifiers,
                name,
                type_parameters: Some(Arc::new(type_parameters)),
                heritage_clauses: Some(Arc::new(heritage_clauses)),
                members: Arc::new(members),
            }),
        ))
    }

    fn new_heritage_clause(&self, token: SyntaxKind, types: NodeList) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::HeritageClause,
            NodeData::HeritageClause(HeritageClauseData {
                token,
                types: Arc::new(types),
            }),
        ))
    }

    fn new_property_signature_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        question_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::PropertySignature,
            NodeData::PropertySignatureDeclaration(PropertySignatureDeclarationData {
                modifiers,
                name,
                postfix_token: question_token,
                type_node: type_node.unwrap_or_else(|| token_node(SyntaxKind::UnknownKeyword)),
                initializer: initializer
                    .unwrap_or_else(|| token_node(SyntaxKind::UnknownKeyword)),
            }),
        ))
    }

    fn new_constructor_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        type_parameters: Option<Arc<NodeList>>,
        parameters: NodeList,
        type_node: Option<Arc<Node>>,
        full_signature: Option<Arc<Node>>,
        body: Option<Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::Constructor,
            NodeData::ConstructorDeclaration(ConstructorDeclarationData {
                modifiers,
                type_parameters,
                parameters: Arc::new(parameters),
                type_node,
                full_signature,
                body,
            }),
        ))
    }

    fn new_variable_statement(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        declaration_list: Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::VariableStatement,
            NodeData::VariableStatement(VariableStatementData {
                modifiers,
                declaration_list,
            }),
        ))
    }

    fn new_variable_declaration_list(
        &self,
        declarations: NodeList,
        _flags: tsox_frontend::ast::NodeFlags,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::VariableDeclarationList,
            NodeData::VariableDeclarationList(VariableDeclarationListData {
                declarations: Arc::new(declarations),
            }),
        ))
    }

    fn new_variable_declaration(
        &self,
        name: Arc<Node>,
        exclamation_token: Option<Arc<Node>>,
        type_node: Option<Arc<Node>>,
        initializer: Option<Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::VariableDeclaration,
            NodeData::VariableDeclaration(VariableDeclarationData {
                name,
                exclamation_token,
                type_node,
                initializer,
            }),
        ))
    }

    fn new_expression_statement(&self, expression: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ExpressionStatement,
            NodeData::ExpressionStatement(ExpressionStatementData { expression }),
        ))
    }

    fn new_export_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        is_type_only: bool,
        export_clause: Arc<Node>,
        module_specifier: Option<Arc<Node>>,
        attributes: Option<Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ExportDeclaration,
            NodeData::ExportDeclaration(ExportDeclarationData {
                modifiers,
                is_type_only,
                export_clause: Some(export_clause),
                module_specifier,
                attributes,
            }),
        ))
    }

    fn new_named_exports(&self, elements: NodeList) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::NamedExports,
            NodeData::NamedExports(NamedExportsData {
                elements: Arc::new(elements),
            }),
        ))
    }

    fn new_export_specifier(
        &self,
        is_type_only: bool,
        property_name: Option<Arc<Node>>,
        name: Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ExportSpecifier,
            NodeData::ExportSpecifier(ExportSpecifierData {
                is_type_only,
                property_name,
                name,
            }),
        ))
    }

    fn new_module_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        keyword: SyntaxKind,
        name: Arc<Node>,
        attributes: Option<Arc<Node>>,
        body: Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ModuleDeclaration,
            NodeData::ModuleDeclaration(ModuleDeclarationData {
                modifiers,
                keyword,
                name,
                attributes,
                body: Some(body),
            }),
        ))
    }

    fn new_module_block(&self, statements: NodeList) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ModuleBlock,
            NodeData::ModuleBlock(ModuleBlockData {
                statements: Arc::new(statements),
            }),
        ))
    }

    fn new_type_alias_declaration(
        &self,
        modifiers: Option<Arc<ModifierList>>,
        name: Arc<Node>,
        type_parameters: NodeList,
        type_node: Option<Arc<Node>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TypeAliasDeclaration,
            NodeData::TypeAliasDeclaration(TypeAliasDeclarationData {
                modifiers,
                name,
                type_parameters: Some(Arc::new(type_parameters)),
                type_node: type_node.unwrap_or_else(|| token_node(SyntaxKind::UnknownKeyword)),
            }),
        ))
    }

    fn new_type_reference_node(
        &self,
        type_name: &Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TypeReference,
            NodeData::TypeReferenceNode(TypeReferenceNodeData {
                type_name: Arc::clone(type_name),
                type_arguments,
            }),
        ))
    }

    fn new_property_access_expression(
        &self,
        expression: &Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        name: &Arc<Node>,
        _flags: tsox_frontend::ast::NodeFlags,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::PropertyAccessExpression,
            NodeData::PropertyAccessExpression(tsox_frontend::ast::node_data_generated::PropertyAccessExpressionData {
                expression: Arc::clone(expression),
                question_dot_token,
                name: Arc::clone(name),
            }),
        ))
    }

    fn new_indexed_access_type_node(
        &self,
        object_type: &Arc<Node>,
        index_type: &Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::IndexedAccessType,
            NodeData::IndexedAccessTypeNode(tsox_frontend::ast::node_data_generated::IndexedAccessTypeNodeData {
                object_type: Arc::clone(object_type),
                index_type: Arc::clone(index_type),
            }),
        ))
    }

    fn new_parenthesized_type_node(&self, type_node: &Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ParenthesizedType,
            NodeData::ParenthesizedTypeNode(tsox_frontend::ast::node_data_generated::ParenthesizedTypeNodeData {
                type_node: Arc::clone(type_node),
            }),
        ))
    }

    fn new_type_query_node(&self, expr_name: &Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TypeQuery,
            NodeData::TypeQueryNode(TypeQueryNodeData {
                expr_name: Arc::clone(expr_name),
                type_arguments: None,
            }),
        ))
    }

    fn new_literal_type_node(&self, literal: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::LiteralType,
            NodeData::LiteralTypeNode(tsox_frontend::ast::node_data_generated::LiteralTypeNodeData { literal }),
        ))
    }

    fn new_element_access_expression(
        &self,
        expression: &Arc<Node>,
        question_dot_token: Option<Arc<Node>>,
        argument_expression: &Arc<Node>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ElementAccessExpression,
            NodeData::ElementAccessExpression(tsox_frontend::ast::node_data_generated::ElementAccessExpressionData {
                expression: Arc::clone(expression),
                question_dot_token,
                argument_expression: Arc::clone(argument_expression),
            }),
        ))
    }

    fn update_type_reference_node(
        &self,
        node: &Arc<Node>,
        _type_name: &Arc<Node>,
        _type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> {
        Arc::clone(node)
    }

    fn update_import_type_node(
        &self,
        node: &Arc<Node>,
        _argument: Option<&Arc<Node>>,
        _qualifier: Option<&Arc<Node>>,
        _type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> {
        Arc::clone(node)
    }

    fn add_synthetic_leading_comment(
        &self,
        node: &Arc<Node>,
        _kind: SyntaxKind,
        _text: &str,
    ) -> Arc<Node> {
        Arc::clone(node)
    }

    fn new_keyword_type_node_ex(&self, kind: SyntaxKind) -> Arc<Node> {
        token_node(kind)
    }

    fn update_class_declaration(
        &self,
        node: &Arc<Node>,
        modifiers: Option<Arc<ModifierList>>,
        name: Option<Arc<Node>>,
        type_parameters: Option<Arc<NodeList>>,
        heritage_clauses: Option<Arc<NodeList>>,
        members: Arc<NodeList>,
    ) -> Arc<Node> {
        if let NodeData::ClassDeclaration(d) = &node.data {
            return Arc::new(Node::new(
                SyntaxKind::ClassDeclaration,
                NodeData::ClassDeclaration(ClassDeclarationData {
                    modifiers: modifiers.or_else(|| d.modifiers.clone()),
                    name: name.or_else(|| d.name.clone()),
                    type_parameters: type_parameters.or_else(|| d.type_parameters.clone()),
                    heritage_clauses: heritage_clauses.or_else(|| d.heritage_clauses.clone()),
                    members,
                }),
            ));
        }
        Arc::clone(node)
    }
}

pub trait R22K6NodeBuilderExt {
    fn signature_to_signature_declaration_helper(
        &mut self,
        signature: &Arc<Signature>,
        kind: SyntaxKind,
        options: Option<&SignatureToSignatureDeclarationOptions>,
    ) -> Arc<Node>;
    fn index_info_to_index_signature_declaration_helper(
        &mut self,
        info: &crate::checker::types_impl_chunk_3::IndexInfo,
        _options: Option<&Arc<Node>>,
    ) -> Arc<Node>;
    fn add_property_to_element_list(
        &mut self,
        property: &Arc<tsox_frontend::ast::Symbol>,
        elements: Vec<Arc<Node>>,
    ) -> Vec<Arc<Node>>;
}

impl<'a> R22K6NodeBuilderExt for NodeBuilderImpl<'a> {
    fn signature_to_signature_declaration_helper(
        &mut self,
        signature: &Arc<Signature>,
        kind: SyntaxKind,
        options: Option<&SignatureToSignatureDeclarationOptions>,
    ) -> Arc<Node> {
        let _ = options;
        if let Some(decl) = signature.declaration.as_ref() {
            return Arc::clone(decl);
        }
        token_node(kind)
    }

    fn index_info_to_index_signature_declaration_helper(
        &mut self,
        info: &crate::checker::types_impl_chunk_3::IndexInfo,
        _options: Option<&Arc<Node>>,
    ) -> Arc<Node> {
        let _ = info;
        token_node(SyntaxKind::IndexSignature)
    }

    fn add_property_to_element_list(
        &mut self,
        property: &Arc<tsox_frontend::ast::Symbol>,
        mut elements: Vec<Arc<Node>>,
    ) -> Vec<Arc<Node>> {
        let ch = unsafe { &mut *crate::checker::mig::m2c_5::r26k4_defs::builder_checker_ptr() };
        let t = ch.get_type_of_symbol(property);
        let type_node = self.type_to_type_node(&t);
        let node = <NodeFactoryStub as R22K6NodeFactoryExt>::new_property_signature_declaration(
            &self.f,
            None,
            crate::checker::mig::m2c::r19k8_defs::new_identifier(&property.name),
            None,
            type_node,
            None,
        );
        elements.push(node);
        elements
    }
}

pub fn filter_type_ext(
    c: &mut Checker,
    t: &Arc<Type>,
    f: &mut dyn FnMut(&mut Checker, &Arc<Type>) -> bool,
) -> Arc<Type> {
    if t.flags.intersects(TypeFlags::Union) {
        let types = t.types().unwrap_or(&[]).to_vec();
        let filtered: Vec<Arc<Type>> = types.iter().filter(|u| f(c, u)).cloned().collect();
        let same = types.len() == filtered.len()
            && types
                .iter()
                .zip(filtered.iter())
                .all(|(a, b)| Arc::ptr_eq(a, b));
        if same {
            return Arc::clone(t);
        }
        let origin = t.as_union_type().and_then(|u| u.origin.clone());
        let mut new_origin: Option<Arc<Type>> = None;
        if let Some(origin) = &origin {
            if origin.flags.intersects(TypeFlags::Union) {
                let origin_types = origin.types().unwrap_or(&[]).to_vec();
                let origin_filtered: Vec<Arc<Type>> = origin_types
                    .iter()
                    .filter(|u| u.flags.intersects(TypeFlags::Union) || f(c, u))
                    .cloned()
                    .collect();
                if origin_types.len() - origin_filtered.len() == types.len() - filtered.len() {
                    if origin_filtered.len() == 1 {
                        return Arc::clone(&origin_filtered[0]);
                    }
                    new_origin = Some(c.new_union_type(ObjectFlags::None, &origin_filtered));
                }
            }
        }
        let object_flags =
            t.object_flags & (ObjectFlags::PrimitiveUnion | ObjectFlags::ContainsIntersections);
        return c.get_union_type_from_sorted_list(filtered, object_flags, None, new_origin.as_ref());
    }
    if t.flags.intersects(TypeFlags::Never) || f(c, t) {
        return Arc::clone(t);
    }
    c.never_type()
}

pub fn map_type_ext(
    c: &mut Checker,
    t: &Arc<Type>,
    f: &mut dyn FnMut(&mut Checker, &Arc<Type>) -> Option<Arc<Type>>,
) -> Option<Arc<Type>> {
    if t.flags.intersects(TypeFlags::Never) {
        return Some(Arc::clone(t));
    }
    if !t.flags.intersects(TypeFlags::Union) {
        return f(c, t);
    }
    let mut types: Vec<Arc<Type>> = t.types().unwrap_or(&[]).to_vec();
    let origin = t.as_union_type().and_then(|u| u.origin.clone());
    if let Some(origin) = &origin {
        if origin.flags.intersects(TypeFlags::Union) {
            types = origin.types().unwrap_or(&[]).to_vec();
        }
    }
    let mut mapped_types: Vec<Arc<Type>> = Vec::with_capacity(16);
    let mut changed = false;
    for s in &types {
        let mapped = if s.flags.intersects(TypeFlags::Union) {
            map_type_ext(c, s, f)
        } else {
            f(c, s)
        };
        match mapped {
            Some(mapped) => {
                if !Arc::ptr_eq(&mapped, s) {
                    changed = true;
                }
                mapped_types.push(mapped);
            }
            None => changed = true,
        }
    }
    if changed {
        if mapped_types.is_empty() {
            return None;
        }
        return Some(c.get_union_type(mapped_types));
    }
    Some(Arc::clone(t))
}
