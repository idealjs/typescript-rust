#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_core::core::text::TextRange;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::{self, Node, SourceFile, SyntaxKind};

use crate::lsp::lsproto_lsp::Range;

pub fn range_contains_range(r1: TextRange, r2: TextRange) -> bool { ::tsox_core::fntrace::enter("range_contains_range"); 
    start_end_contains_range(r1.pos as usize, r1.end as usize, r2)
}

pub fn start_end_contains_range(start: usize, end: usize, text_range: TextRange) -> bool { ::tsox_core::fntrace::enter("start_end_contains_range"); 
    start <= text_range.pos as usize && end >= text_range.end as usize
}

impl crate::ls::language_service::LanguageService {
    pub fn create_lsp_range_from_node(
        &self,
        node: &Arc<Node>,
        file: &Arc<SourceFile>,
    ) -> (Range, u32) { ::tsox_core::fntrace::enter("create_lsp_range_from_node"); 
        self.m5x_create_lsp_range_from_bounds(
            tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(node, file, false),
            node.end(),
            file,
        )
    }

    pub fn create_lsp_range_from_node_for_feature(
        &self,
        node: &Arc<Node>,
        file: &Arc<SourceFile>,
        feature: u32,
    ) -> (Range, u32) { ::tsox_core::fntrace::enter("create_lsp_range_from_node_for_feature"); 
        let text_range = create_range_from_node(node, file);
        let script_view = crate::mig::m5u_conv::SourceFileScriptView {
            file: Arc::clone(file),
        };
        m5u_converters().to_lsp_range_for_feature(&script_view, text_range, feature)
    }

    pub fn m5x_create_lsp_range_from_bounds(
        &self,
        start: usize,
        end: usize,
        file: &Arc<SourceFile>,
    ) -> (Range, u32) { ::tsox_core::fntrace::enter("m5x_create_lsp_range_from_bounds"); 
        let script_view = crate::mig::m5u_conv::SourceFileScriptView {
            file: Arc::clone(file),
        };
        m5u_converters().to_lsp_range(&script_view, TextRange::new(start, end))
    }

    pub fn create_lsp_range_from_range_m5x(
        &self,
        text_range: TextRange,
        script: &dyn crate::ls::lsconv_converters::Script,
    ) -> (Range, u32) { ::tsox_core::fntrace::enter("create_lsp_range_from_range_m5x"); 
        let m5u_script = crate::mig::m5u_conv::OriginalTextScript {
            file_name: script.file_name().to_string(),
            text: script.text().to_string(),
        };
        m5u_converters().to_lsp_range(
            &m5u_script,
            TextRange::new(text_range.pos as usize, text_range.end as usize),
        )
    }

    pub fn create_lsp_position_m5x(
        &self,
        position: usize,
        file: &Arc<SourceFile>,
    ) -> (crate::lsp::lsproto_lsp_basic::Position, u32) { ::tsox_core::fntrace::enter("create_lsp_position_m5x"); 
        let script_view = crate::mig::m5u_conv::SourceFileScriptView {
            file: Arc::clone(file),
        };
        m5u_converters().to_lsp_position(&script_view, position)
    }
}

fn m5u_converters() -> crate::mig::m5u_conv::M5uConverters { ::tsox_core::fntrace::enter("m5u_converters"); 
    crate::mig::m5u_conv::new_converters(
        crate::ls::lsconv_converters::PositionEncodingKind::Utf16,
        Box::new(crate::ls::lsconv_linemap::compute_lsp_line_starts),
    )
}

pub fn create_range_from_node(node: &Arc<Node>, file: &Arc<SourceFile>) -> TextRange { ::tsox_core::fntrace::enter("create_range_from_node"); 
    TextRange::new(
        tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(node, file, false),
        node.end(),
    )
}

pub fn quote(
    file: &Arc<SourceFile>,
    preferences: &crate::ls::lsutil_user_preferences::UserPreferences,
    text: &str,
) -> String { ::tsox_core::fntrace::enter("quote"); 
    let quote_preference =
        crate::ls::lsutil_utilities::get_quote_preference(file, preferences);
    let mut quoted = tsox_core::core::mig::m3j::stringify_json(&text, "", "").unwrap_or_default();
    if quote_preference == crate::ls::lsutil_user_preferences::QuotePreference::Single {
        quoted = format!(
            "'{}'",
            strip_quotes(&quote_replacer(quoted))
        );
    }
    quoted
}

fn quote_replacer(text: String) -> String { ::tsox_core::fntrace::enter("quote_replacer"); 
    text.replace('\'', "\\'")
}

pub fn is_type_keyword(kind: SyntaxKind) -> bool { ::tsox_core::fntrace::enter("is_type_keyword"); 
    matches!(
        kind,
        SyntaxKind::AnyKeyword
            | SyntaxKind::AssertsKeyword
            | SyntaxKind::BigIntKeyword
            | SyntaxKind::BooleanKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::InferKeyword
            | SyntaxKind::KeyOfKeyword
            | SyntaxKind::NeverKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::NumberKeyword
            | SyntaxKind::ObjectKeyword
            | SyntaxKind::ReadonlyKeyword
            | SyntaxKind::StringKeyword
            | SyntaxKind::SymbolKeyword
            | SyntaxKind::TypeOfKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::VoidKeyword
            | SyntaxKind::UndefinedKeyword
            | SyntaxKind::UniqueKeyword
            | SyntaxKind::UnknownKeyword
    )
}

pub fn is_separator(node: &Arc<Node>, candidate: Option<&Arc<Node>>) -> bool { ::tsox_core::fntrace::enter("is_separator"); 
    candidate.map_or(false, |candidate| {
        node.parent().map_or(false, |parent| {
            candidate.kind == SyntaxKind::CommaToken
                || (candidate.kind == SyntaxKind::SemicolonToken
                    && parent.kind == SyntaxKind::ObjectLiteralExpression)
        })
    })
}

pub fn is_literal_name_of_property_declaration_or_index_access(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_literal_name_of_property_declaration_or_index_access"); 
    let Some(parent) = node.parent() else { return false };
    match parent.kind {
        SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::EnumMember
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::MethodSignature
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::ModuleDeclaration => {
            ast::get_name_of_declaration(&parent).map_or(false, |name| Arc::ptr_eq(&name, node))
        }
        SyntaxKind::ElementAccessExpression => {
            crate::ls::mig::m5x_3::element_access_expression_argument_expression(&parent)
                .map_or(false, |arg| Arc::ptr_eq(&arg, node))
        }
        SyntaxKind::ComputedPropertyName => true,
        SyntaxKind::LiteralType => parent
            .parent()
            .map_or(false, |grand| grand.kind == SyntaxKind::IndexedAccessType),
        _ => false,
    }
}

pub fn is_object_binding_element_without_property_name(binding_element: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_object_binding_element_without_property_name"); 
    binding_element.kind == SyntaxKind::BindingElement
        && binding_element
            .parent()
            .map_or(false, |p| p.kind == SyntaxKind::ObjectBindingPattern)
        && crate::ls::mig::m5x_3::binding_element_name(binding_element)
            .map_or(false, |name| name.kind == SyntaxKind::Identifier)
        && crate::ls::mig::m5x_3::binding_element_property_name(binding_element).is_none()
}

pub fn is_right_side_of_property_access(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_right_side_of_property_access"); 
    node.parent().map_or(false, |parent| {
        parent.kind == SyntaxKind::PropertyAccessExpression
            && crate::ls::mig::m5x_3::access_expression_name(&parent).map_or(false, |name| Arc::ptr_eq(&name, node))
    })
}

pub fn is_static_symbol(symbol: &tsox_frontend::ast::Symbol) -> bool { ::tsox_core::fntrace::enter("is_static_symbol"); 
    symbol
        .value_declaration
        .as_ref()
        .map_or(false, |decl| decl.syntactic_modifier_flags().contains(ast::ModifierFlags::Static))
}

pub fn is_implementation(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_implementation"); 
    if node.flags.contains(ast::NodeFlags::Ambient) {
        return !(node.kind == SyntaxKind::InterfaceDeclaration
            || node.kind == SyntaxKind::TypeAliasDeclaration);
    }
    if ast::mig::m3g_3::is_variable_like(node) {
        return ast::mig::m3f_2::has_initializer(node);
    }
    if ast::is_function_like_declaration(node) {
        return crate::ls::mig::m5x_3::node_body(node).is_some();
    }
    ast::is_class_like(node) || ast::is_module_or_enum_declaration(node)
}

pub fn is_implementation_expression(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_implementation_expression"); 
    match node.kind {
        SyntaxKind::ParenthesizedExpression => crate::ls::mig::m5x_3::parenthesized_expression_expression(node)
            .map_or(false, |expr| is_implementation_expression(&expr)),
        SyntaxKind::ArrowFunction
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ObjectLiteralExpression
        | SyntaxKind::ClassExpression
        | SyntaxKind::ArrayLiteralExpression => true,
        _ => false,
    }
}

pub fn is_readonly_type_operator(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_readonly_type_operator"); 
    node.kind == SyntaxKind::ReadonlyKeyword
        && node
            .parent()
            .map_or(false, |parent| {
                parent.kind == SyntaxKind::TypeOperator
                    && crate::ls::mig::m5x_3::type_operator_node_operator(&parent) == SyntaxKind::ReadonlyKeyword
            })
}

pub fn is_jump_statement_target(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_jump_statement_target"); 
    node.kind == SyntaxKind::Identifier
        && node.parent().map_or(false, |parent| {
            ast::is_break_or_continue_statement(&parent)
                && crate::ls::mig::m5x_3::break_or_continue_statement_label(&parent)
                    .map_or(false, |label| Arc::ptr_eq(&label, node))
        })
}

pub fn is_label_of_labeled_statement(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_label_of_labeled_statement"); 
    node.kind == SyntaxKind::Identifier
        && node.parent().map_or(false, |parent| {
            parent.kind == SyntaxKind::LabeledStatement
                && crate::ls::mig::m5x_3::labeled_statement_label(&parent)
                    .map_or(false, |label| Arc::ptr_eq(&label, node))
        })
}

pub fn find_reference_in_position(
    refs: &[Arc<ast::node_source_file::FileReference>],
    pos: usize,
) -> Option<Arc<ast::node_source_file::FileReference>> { ::tsox_core::fntrace::enter("find_reference_in_position"); 
    refs.iter()
        .find(|r| {
            (r.range.pos as usize) <= pos && pos <= r.range.end as usize
        })
        .cloned()
}

pub fn get_containing_node_if_in_heritage_clause(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_containing_node_if_in_heritage_clause"); 
    if node.kind == SyntaxKind::Identifier
        || node.kind == SyntaxKind::QualifiedName
        || node.kind == SyntaxKind::PropertyAccessExpression
    {
        return node
            .parent()
            .and_then(|parent| get_containing_node_if_in_heritage_clause(&parent));
    }
    if let Some(parent) = node.parent() {
        if (node.kind == SyntaxKind::ExpressionWithTypeArguments
            || node.kind == SyntaxKind::TypeReference)
            && ast::is_heritage_clause(&parent)
        {
            if let Some(grand) = parent.parent() {
                if ast::is_class_like(&grand) || grand.kind == SyntaxKind::InterfaceDeclaration {
                    return Some(grand);
                }
            }
        }
    }
    None
}

pub fn access_expression_expression(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("access_expression_expression"); 
    match &node.data {
        NodeData::PropertyAccessExpression(d) => Some(Arc::clone(&d.expression)),
        NodeData::ElementAccessExpression(d) => Some(Arc::clone(&d.expression)),
        NodeData::CallExpression(d) => Some(Arc::clone(&d.expression)),
        NodeData::NewExpression(d) => Some(Arc::clone(&d.expression)),
        _ => None,
    }
}

pub fn access_expression_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("access_expression_name"); 
    match &node.data {
        NodeData::PropertyAccessExpression(d) => Some(Arc::clone(&d.name)),
        _ => None,
    }
}

pub fn element_access_expression_argument_expression(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("element_access_expression_argument_expression"); 
    match &node.data {
        NodeData::ElementAccessExpression(d) => Some(Arc::clone(&d.argument_expression)),
        _ => None,
    }
}

pub fn binding_element_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("binding_element_name"); 
    match &node.data {
        NodeData::BindingElement(d) => d.name.clone(),
        _ => None,
    }
}

pub fn binding_element_property_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("binding_element_property_name"); 
    match &node.data {
        NodeData::BindingElement(d) => d.property_name.clone(),
        _ => None,
    }
}

pub fn node_body(node: &Node) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("node_body"); 
    match &node.data {
        NodeData::FunctionDeclaration(d) => d.body.clone(),
        NodeData::FunctionExpression(d) => Some(Arc::clone(&d.body)),
        NodeData::ConstructorDeclaration(d) => d.body.clone(),
        NodeData::MethodDeclaration(d) => d.body.clone(),
        NodeData::GetAccessorDeclaration(d) => d.body.clone(),
        NodeData::SetAccessorDeclaration(d) => d.body.clone(),
        NodeData::ArrowFunction(d) => Some(Arc::clone(&d.body)),
        _ => None,
    }
}

pub fn parenthesized_expression_expression(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("parenthesized_expression_expression"); 
    match &node.data {
        NodeData::ParenthesizedExpression(d) => Some(Arc::clone(&d.expression)),
        _ => None,
    }
}

pub fn type_operator_node_operator(node: &Arc<Node>) -> SyntaxKind { ::tsox_core::fntrace::enter("type_operator_node_operator"); 
    match &node.data {
        NodeData::TypeOperatorNode(d) => d.operator,
        _ => node.kind,
    }
}

pub fn labeled_statement_label(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("labeled_statement_label"); 
    match &node.data {
        NodeData::LabeledStatement(d) => Some(Arc::clone(&d.label)),
        _ => None,
    }
}

pub fn break_or_continue_statement_label(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("break_or_continue_statement_label"); 
    match &node.data {
        NodeData::BreakStatement(d) => d.label.clone(),
        NodeData::ContinueStatement(d) => d.label.clone(),
        _ => None,
    }
}

pub fn call_expression_expression(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("call_expression_expression"); 
    match &node.data {
        NodeData::CallExpression(d) => Some(Arc::clone(&d.expression)),
        _ => None,
    }
}

pub fn call_expression_arguments(node: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("call_expression_arguments"); 
    match &node.data {
        NodeData::CallExpression(d) => d.arguments.nodes.clone(),
        _ => Vec::new(),
    }
}

pub fn call_or_new_expression_expression(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("call_or_new_expression_expression"); 
    match &node.data {
        NodeData::CallExpression(d) => Some(Arc::clone(&d.expression)),
        NodeData::NewExpression(d) => Some(Arc::clone(&d.expression)),
        _ => None,
    }
}

pub fn call_or_new_expression_type_arguments(node: &Arc<Node>) -> Option<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("call_or_new_expression_type_arguments"); 
    match &node.data {
        NodeData::CallExpression(d) => d.type_arguments.as_ref().map(|l| l.nodes.clone()),
        NodeData::NewExpression(d) => d.type_arguments.as_ref().map(|l| l.nodes.clone()),
        _ => None,
    }
}

pub fn binary_expression_operator_token(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("binary_expression_operator_token"); 
    match &node.data {
        NodeData::BinaryExpression(d) => Some(Arc::clone(&d.operator_token)),
        _ => None,
    }
}

pub fn binary_expression_left(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("binary_expression_left"); 
    match &node.data {
        NodeData::BinaryExpression(d) => Some(Arc::clone(&d.left)),
        _ => None,
    }
}

pub fn binary_expression_right(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("binary_expression_right"); 
    match &node.data {
        NodeData::BinaryExpression(d) => Some(Arc::clone(&d.right)),
        _ => None,
    }
}

pub fn module_declaration_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("module_declaration_name"); 
    match &node.data {
        NodeData::ModuleDeclaration(d) => Some(Arc::clone(&d.name)),
        _ => None,
    }
}

pub fn module_declaration_body(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("module_declaration_body"); 
    match &node.data {
        NodeData::ModuleDeclaration(d) => d.body.clone(),
        _ => None,
    }
}

pub fn import_equals_declaration_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_equals_declaration_name"); 
    match &node.data {
        NodeData::ImportEqualsDeclaration(d) => Some(Arc::clone(&d.name)),
        _ => None,
    }
}

pub fn import_equals_declaration_module_reference(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_equals_declaration_module_reference"); 
    match &node.data {
        NodeData::ImportEqualsDeclaration(d) => Some(Arc::clone(&d.module_reference)),
        _ => None,
    }
}

pub fn is_external_module_import_equals_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_external_module_import_equals_declaration"); 
    node.kind == SyntaxKind::ImportEqualsDeclaration
        && import_equals_declaration_module_reference(node)
            .map_or(false, |mr| mr.kind == SyntaxKind::ExternalModuleReference)
}

pub fn is_internal_module_import_equals_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_internal_module_import_equals_declaration"); 
    node.kind == SyntaxKind::ImportEqualsDeclaration
        && !is_external_module_import_equals_declaration(node)
}

pub fn get_external_module_import_equals_declaration_expression(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_external_module_import_equals_declaration_expression"); 
    match &node.data {
        NodeData::ImportEqualsDeclaration(d) if d.module_reference.kind == SyntaxKind::ExternalModuleReference => d
            .module_reference
            .expression(),
        _ => None,
    }
    .map(|expr| Arc::clone(expr))
}

pub fn heritage_clause_token(node: &Arc<Node>) -> SyntaxKind { ::tsox_core::fntrace::enter("heritage_clause_token"); 
    match &node.data {
        NodeData::HeritageClause(d) => d.token,
        _ => node.kind,
    }
}

pub fn import_clause_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_clause_name"); 
    match &node.data {
        NodeData::ImportClause(d) => d.name.clone(),
        _ => None,
    }
}

pub fn import_clause_named_bindings(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_clause_named_bindings"); 
    match &node.data {
        NodeData::ImportClause(d) => d.named_bindings.clone(),
        _ => None,
    }
}

pub fn import_specifier_property_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_specifier_property_name"); 
    match &node.data {
        NodeData::ImportSpecifier(d) => d.property_name.clone(),
        _ => None,
    }
}

pub fn export_specifier_property_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("export_specifier_property_name"); 
    match &node.data {
        NodeData::ExportSpecifier(d) => d.property_name.clone(),
        _ => None,
    }
}

pub fn import_specifier_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_specifier_name"); 
    match &node.data {
        NodeData::ImportSpecifier(d) => Some(Arc::clone(&d.name)),
        _ => None,
    }
}

pub fn export_specifier_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("export_specifier_name"); 
    node.name().cloned()
}

pub fn namespace_import_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("namespace_import_name"); 
    match &node.data {
        NodeData::NamespaceImport(d) => Some(Arc::clone(&d.name)),
        _ => None,
    }
}

pub fn namespace_export_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("namespace_export_name"); 
    match &node.data {
        NodeData::NamespaceExport(d) => Some(Arc::clone(&d.name)),
        _ => None,
    }
}

pub fn named_imports_elements(node: &Arc<Node>) -> Option<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("named_imports_elements"); 
    match &node.data {
        NodeData::NamedImports(d) => Some(d.elements.nodes.clone()),
        _ => None,
    }
}

pub fn named_exports_elements(node: &Arc<Node>) -> Option<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("named_exports_elements"); 
    match &node.data {
        NodeData::NamedExports(d) => Some(d.elements.nodes.clone()),
        _ => None,
    }
}

pub fn import_or_export_specifier_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_or_export_specifier_name"); 
    import_specifier_property_name(node)
        .or_else(|| export_specifier_property_name(node))
        .or_else(|| node.name().cloned())
}

pub fn import_declaration_import_clause(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_declaration_import_clause"); 
    ast::mig::m3b::import_clause(node).cloned()
}

pub fn import_declaration_module_specifier(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_declaration_module_specifier"); 
    ast::mig::m3b::module_specifier(node).cloned()
}

pub fn export_declaration_export_clause(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("export_declaration_export_clause"); 
    match &node.data {
        NodeData::ExportDeclaration(d) => d.export_clause.clone(),
        _ => None,
    }
}

pub fn export_declaration_module_specifier(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("export_declaration_module_specifier"); 
    match &node.data {
        NodeData::ExportDeclaration(d) => d.module_specifier.clone(),
        _ => None,
    }
}

pub fn import_or_export_declaration_module_specifier(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_or_export_declaration_module_specifier"); 
    ast::mig::m3b::module_specifier(node).cloned()
}

pub fn variable_declaration_list_declarations(node: &Arc<Node>) -> Option<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("variable_declaration_list_declarations"); 
    match &node.data {
        NodeData::VariableDeclarationList(d) => Some(d.declarations.nodes.clone()),
        _ => None,
    }
}

pub fn variable_declaration_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("variable_declaration_name"); 
    node.name().cloned()
}

pub fn case_clause_expression(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("case_clause_expression"); 
    match &node.data {
        NodeData::CaseOrDefaultClause(d) => Some(d.expression.clone()),
        _ => None,
    }
}

pub fn case_clause_parent_switch_statement(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("case_clause_parent_switch_statement"); 
    node.parent().filter(|p| p.kind == SyntaxKind::SwitchStatement)
}

pub fn switch_statement_expression(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("switch_statement_expression"); 
    match &node.data {
        NodeData::SwitchStatement(d) => Some(Arc::clone(&d.expression)),
        _ => None,
    }
}

pub fn template_expression_head(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("template_expression_head"); 
    match &node.data {
        NodeData::TemplateExpression(d) => Some(Arc::clone(&d.head)),
        _ => None,
    }
}

pub fn computed_property_name_expression(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("computed_property_name_expression"); 
    match &node.data {
        NodeData::ComputedPropertyName(d) => Some(Arc::clone(&d.expression)),
        _ => None,
    }
}

pub fn object_literal_element_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("object_literal_element_name"); 
    match node.kind {
        SyntaxKind::PropertyAssignment
        | SyntaxKind::ShorthandPropertyAssignment
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::MethodSignature => node.name().cloned(),
        _ => None,
    }
}

pub fn strip_quotes(text: &str) -> &str { ::tsox_core::fntrace::enter("strip_quotes"); 
    let bytes = text.as_bytes();
    if bytes.len() >= 2
        && matches!(bytes[0], b'"' | b'\'' | b'`')
        && bytes[bytes.len() - 1] == bytes[0]
    {
        &text[1..text.len() - 1]
    } else {
        text
    }
}
