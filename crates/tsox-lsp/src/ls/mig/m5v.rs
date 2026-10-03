#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use serde_json::Value;
use tsox_checker::checker::Checker;
use tsox_frontend::ast::{self, Node, Symbol, SyntaxKind};

use crate::ls::lsutil::{
    IndentStyle, OrganizeImportsTypeOrder, SemicolonPreference, UserPreferences,
    new_default_user_preferences,
};
use crate::ls::lsutil_organize_imports_comparers::{StatementComparer, StringComparer};
use crate::ls::lsutil_symbol_display::{ScriptElementKind, ScriptElementKindModifier};
use tsox_frontend::ast::node_data_generated::{
    CallExpressionData, ImportClauseData, ImportDeclarationData, NodeData, VariableDeclarationData,
    VariableDeclarationListData, VariableStatementData,
};
use tsox_frontend::ast::NodeList;

pub trait M5vNodeExt {
    fn import_clause(&self) -> Option<&Arc<Node>>;
    fn named_bindings(&self) -> Option<&Arc<Node>>;
    fn elements(&self) -> &NodeList;
    fn is_type_only(&self) -> bool;
    fn as_variable_statement(&self) -> &VariableStatementData;
    fn as_variable_declaration_list(&self) -> &VariableDeclarationListData;
    fn as_variable_declaration(&self) -> &VariableDeclarationData;
    fn as_call_expression(&self) -> &CallExpressionData;
    fn initializer(&self) -> Option<&Arc<Node>>;
}

macro_rules! m5v_as_data {
    ($name:ident, $variant:ident, $ty:ty) => {
        fn $name(&self) -> &$ty {
            match &self.data {
                NodeData::$variant(d) => d,
                _ => panic!(concat!("As", stringify!($variant), " on wrong node kind")),
            }
        }
    };
}

impl M5vNodeExt for Node {
    fn import_clause(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("import_clause"); 
        match &self.data {
            NodeData::ImportDeclaration(d) => d.import_clause.as_ref(),
            _ => None,
        }
    }

    fn named_bindings(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("named_bindings"); 
        match &self.data {
            NodeData::ImportClause(d) => d.named_bindings.as_ref(),
            _ => None,
        }
    }

    fn elements(&self) -> &NodeList { ::tsox_core::fntrace::enter("elements"); 
        match &self.data {
            NodeData::NamedImports(d) => &d.elements,
            NodeData::NamedExports(d) => &d.elements,
            _ => panic!("Elements on wrong node kind"),
        }
    }

    fn is_type_only(&self) -> bool { ::tsox_core::fntrace::enter("is_type_only"); 
        match &self.data {
            NodeData::ImportEqualsDeclaration(d) => d.is_type_only,
            NodeData::ImportSpecifier(d) => d.is_type_only,
            NodeData::ExportSpecifier(d) => d.is_type_only,
            NodeData::ExportDeclaration(d) => d.is_type_only,
            _ => false,
        }
    }

    m5v_as_data!(as_variable_statement, VariableStatement, VariableStatementData);
    m5v_as_data!(
        as_variable_declaration_list,
        VariableDeclarationList,
        VariableDeclarationListData
    );
    m5v_as_data!(as_variable_declaration, VariableDeclaration, VariableDeclarationData);
    m5v_as_data!(as_call_expression, CallExpression, CallExpressionData);

    fn initializer(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("initializer"); 
        match &self.data {
            NodeData::VariableDeclaration(d) => d.initializer.as_ref(),
            _ => None,
        }
    }
}

pub fn parse_indent_style(v: &Value) -> IndentStyle { ::tsox_core::fntrace::enter("parse_indent_style"); 
    match v {
        Value::String(s) => match s.to_lowercase().as_str() {
            "none" => IndentStyle::None,
            "block" => IndentStyle::Block,
            "smart" => IndentStyle::Smart,
            _ => IndentStyle::Smart,
        },
        Value::Number(n) => match n.as_i64() {
            Some(0) => IndentStyle::None,
            Some(1) => IndentStyle::Block,
            Some(2) => IndentStyle::Smart,
            _ => IndentStyle::Smart,
        },
        _ => IndentStyle::Smart,
    }
}

pub fn parse_semicolon_preference(v: &Value) -> SemicolonPreference { ::tsox_core::fntrace::enter("parse_semicolon_preference"); 
    if let Value::String(s) = v {
        match s.to_lowercase().as_str() {
            "insert" => return SemicolonPreference::Insert,
            "remove" => return SemicolonPreference::Remove,
            _ => {}
        }
    }
    SemicolonPreference::Ignore
}

pub fn remove_diacritics(s: &str) -> String { ::tsox_core::fntrace::enter("remove_diacritics"); 
    use unicode_normalization::UnicodeNormalization;
    s.nfd().filter(|r| !is_combining_mark(*r)).collect()
}

fn is_combining_mark(r: char) -> bool { ::tsox_core::fntrace::enter("is_combining_mark"); 
    matches!(
        unicode_general_category::get_general_category(r),
        unicode_general_category::GeneralCategory::NonspacingMark
    )
}

fn measure_node_sortedness(arr: &[Arc<Node>], comparer: &StatementComparer) -> i32 { ::tsox_core::fntrace::enter("measure_node_sortedness"); 
    let mut count = 0i32;
    for j in 0..arr.len().saturating_sub(1) {
        if comparer(&arr[j], &arr[j + 1]) > 0 {
            count += 1;
        }
    }
    count
}

pub fn get_module_specifier_expression(declaration: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_module_specifier_expression"); 
    match declaration.kind {
        SyntaxKind::ImportEqualsDeclaration => {
            let module_reference = match &declaration.data {
                NodeData::ImportEqualsDeclaration(d) => &d.module_reference,
                _ => return None,
            };
            if module_reference.kind == SyntaxKind::ExternalModuleReference {
                return match &module_reference.data {
                    NodeData::ExternalModuleReference(d) => Some(Arc::clone(&d.expression)),
                    _ => None,
                };
            }
            None
        }
        SyntaxKind::ImportDeclaration => match &declaration.data {
            NodeData::ImportDeclaration(d) => Some(Arc::clone(&d.module_specifier)),
            _ => None,
        },
        SyntaxKind::VariableStatement => {
            let declarations = declaration
                .as_variable_statement()
                .declaration_list
                .as_variable_declaration_list()
                .declarations
                .nodes
                .clone();
            if let Some(first) = declarations.first() {
                if let Some(initializer) = first.initializer() {
                    if initializer.kind == SyntaxKind::CallExpression {
                        let call_expr = initializer.as_call_expression();
                        if !call_expr.arguments.nodes.is_empty() {
                            return Some(Arc::clone(&call_expr.arguments.nodes[0]));
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}

pub struct NamedImportSortResult {
    pub named_import_comparer: StringComparer,
    pub type_order: OrganizeImportsTypeOrder,
    pub is_sorted: bool,
}

pub fn detect_named_import_organization_by_sort(
    original_groups: &[Arc<Node>],
    comparers_to_test: &[StringComparer],
    types_to_test: &[OrganizeImportsTypeOrder],
) -> Option<(StringComparer, OrganizeImportsTypeOrder, bool)> { ::tsox_core::fntrace::enter("detect_named_import_organization_by_sort"); 
    let result =
        detect_named_import_organization_by_sort_inner(original_groups, comparers_to_test, types_to_test)?;
    Some((result.named_import_comparer, result.type_order, result.is_sorted))
}

fn named_imports_elements_of(import_decl: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("named_imports_elements_of"); 
    import_decl
        .import_clause()
        .and_then(|clause| clause.named_bindings())
        .filter(|bindings| bindings.kind == SyntaxKind::NamedImports)
        .map(|bindings| bindings.elements().map(|nl| nl.nodes.clone()).unwrap_or_default())
        .unwrap_or_default()
}

fn detect_named_import_organization_by_sort_inner(
    original_groups: &[Arc<Node>],
    comparers_to_test: &[StringComparer],
    types_to_test: &[OrganizeImportsTypeOrder],
) -> Option<NamedImportSortResult> { ::tsox_core::fntrace::enter("detect_named_import_organization_by_sort_inner"); 
    let mut both_named_imports = false;
    let mut import_decls_with_named: Vec<Arc<Node>> = Vec::new();

    for imp in original_groups {
        let elements = named_imports_elements_of(imp);
        if elements.is_empty() {
            continue;
        }
        if !both_named_imports {
            let mut has_type_only = false;
            let mut has_regular = false;
            for elem in &elements {
                if elem.is_type_only() {
                    has_type_only = true;
                } else {
                    has_regular = true;
                }
            }
            if has_type_only && has_regular {
                both_named_imports = true;
            }
        }
        import_decls_with_named.push(Arc::clone(imp));
    }

    if import_decls_with_named.is_empty() {
        return None;
    }

    let named_imports_by_decl: Vec<Vec<Arc<Node>>> = import_decls_with_named
        .iter()
        .map(|imp| named_imports_elements_of(imp))
        .collect();

    if !both_named_imports || types_to_test.is_empty() {
        let names_list: Vec<Vec<String>> = named_imports_by_decl
            .iter()
            .map(|imports| {
                imports
                    .iter()
                    .map(|imp| imp.name().map(|n| n.text().to_string()).unwrap_or_default())
                    .collect()
            })
            .collect();
        let result =
            crate::ls::lsutil_organize_imports_imports::detect_case_sensitivity_by_sort(
                &names_list,
                comparers_to_test,
            );
        let comparer = result
            .comparer
            .expect("at least one comparer to test is required");
        let type_order = if types_to_test.len() == 1 {
            types_to_test[0]
        } else {
            OrganizeImportsTypeOrder::Last
        };
        return Some(NamedImportSortResult {
            named_import_comparer: comparer,
            type_order,
            is_sorted: result.is_sorted,
        });
    }

    let mut best_diff = [i32::MAX, i32::MAX, i32::MAX];
    let best_comparer = [
        comparers_to_test[0].clone(),
        comparers_to_test[0].clone(),
        comparers_to_test[0].clone(),
    ];

    for cur_comparer in comparers_to_test {
        let mut curr_diff = [0i32; 3];
        for import_decl in &named_imports_by_decl {
            for type_order in types_to_test {
                let cmp = Arc::clone(cur_comparer);
                let type_order = *type_order;
                let node_cmp: StatementComparer = Box::new(move |n1: &Arc<Node>, n2: &Arc<Node>| {
                    crate::ls::lsutil_organize_imports_imports::compare_import_or_export_specifiers(
                        n1,
                        n2,
                        &cmp,
                        type_order,
                    )
                });
                let diff = measure_node_sortedness(import_decl, &node_cmp);
                curr_diff[type_order_index(type_order)] += diff;
            }
        }
        for type_order in types_to_test {
            let idx = type_order_index(*type_order);
            if curr_diff[idx] < best_diff[idx] {
                best_diff[idx] = curr_diff[idx];
            }
        }
    }

    let mut best_comparer = best_comparer;
    for cur_comparer in comparers_to_test {
        let mut curr_diff = [0i32; 3];
        for import_decl in &named_imports_by_decl {
            for type_order in types_to_test {
                let cmp = Arc::clone(cur_comparer);
                let type_order = *type_order;
                let node_cmp: StatementComparer = Box::new(move |n1: &Arc<Node>, n2: &Arc<Node>| {
                    crate::ls::lsutil_organize_imports_imports::compare_import_or_export_specifiers(
                        n1,
                        n2,
                        &cmp,
                        type_order,
                    )
                });
                let diff = measure_node_sortedness(import_decl, &node_cmp);
                curr_diff[type_order_index(type_order)] += diff;
            }
        }
        for type_order in types_to_test {
            let idx = type_order_index(*type_order);
            if curr_diff[idx] < best_diff[idx] {
                best_comparer[idx] = Arc::clone(cur_comparer);
            }
        }
    }

    for best_type_order in types_to_test {
        let best_idx = type_order_index(*best_type_order);
        let mut is_best = true;
        for test_type_order in types_to_test {
            if best_diff[type_order_index(*test_type_order)] < best_diff[best_idx] {
                is_best = false;
                break;
            }
        }
        if is_best {
            return Some(NamedImportSortResult {
                named_import_comparer: best_comparer[best_idx].clone(),
                type_order: *best_type_order,
                is_sorted: best_diff[best_idx] == 0,
            });
        }
    }

    Some(NamedImportSortResult {
        named_import_comparer: best_comparer[type_order_index(OrganizeImportsTypeOrder::Last)]
            .clone(),
        type_order: OrganizeImportsTypeOrder::Last,
        is_sorted: best_diff[type_order_index(OrganizeImportsTypeOrder::Last)] == 0,
    })
}

fn type_order_index(order: OrganizeImportsTypeOrder) -> usize { ::tsox_core::fntrace::enter("type_order_index"); 
    match order {
        OrganizeImportsTypeOrder::First => 0,
        OrganizeImportsTypeOrder::Inline => 1,
        _ => 2,
    }
}

pub fn get_symbol_kind_of_constructor_property_method_accessor_function_or_var(
    mut type_checker: Option<&mut Checker>,
    symbol: &Arc<Symbol>,
    location: &Arc<Node>,
) -> ScriptElementKind { ::tsox_core::fntrace::enter("get_symbol_kind_of_constructor_property_method_accessor_function_or_var"); 
    let roots: Vec<Arc<Symbol>> = match type_checker.as_deref_mut() {
        Some(ch) => ch.get_root_symbols(symbol),
        None => vec![Arc::clone(symbol)],
    };

    if roots.len() == 1
        && roots[0]
            .combined_local_and_export_symbol_flags()
            .contains(ast::SymbolFlags::Method)
        && match type_checker.as_deref_mut() {
            None => true,
            Some(ch) => {
                let t = ch.get_type_of_symbol_at_location(symbol, location);
                let t = ch.get_non_nullable_type_of(&t);
                !ch.get_call_signatures(&t).is_empty()
            }
        }
    {
        return ScriptElementKind::MemberFunctionElement;
    }

    if let Some(ch) = type_checker.as_deref_mut() {
        if ch.is_undefined_symbol(symbol) {
            return ScriptElementKind::VariableElement;
        }
        if ch.is_arguments_symbol(symbol) {
            return ScriptElementKind::LocalVariableElement;
        }
        if (location.kind == SyntaxKind::ThisKeyword && ast::is_expression(location))
            || ast::is_this_in_type_query(location)
        {
            return ScriptElementKind::ParameterElement;
        }
    }

    let flags = symbol.combined_local_and_export_symbol_flags();
    if flags.contains(ast::SymbolFlags::VARIABLE) {
        if is_first_declaration_of_symbol_parameter(symbol) {
            return ScriptElementKind::ParameterElement;
        } else if symbol
            .value_declaration
            .as_ref()
            .map_or(false, |d| ast::mig::m3g_3::is_var_const(d))
        {
            return ScriptElementKind::ConstElement;
        } else if symbol
            .value_declaration
            .as_ref()
            .map_or(false, |d| ast::mig::m3g_3::is_var_using(d))
        {
            return ScriptElementKind::VariableUsingElement;
        } else if symbol
            .value_declaration
            .as_ref()
            .map_or(false, |d| ast::mig::m3g_3::is_var_await_using(d))
        {
            return ScriptElementKind::VariableAwaitUsingElement;
        } else if symbol
            .declarations
            .iter()
            .any(|d| ast::mig::m3g::is_let(d))
        {
            return ScriptElementKind::LetElement;
        }
        if is_local_variable_or_function(symbol) {
            return ScriptElementKind::LocalVariableElement;
        }
        return ScriptElementKind::VariableElement;
    }
    if flags.contains(ast::SymbolFlags::Function) {
        if is_local_variable_or_function(symbol) {
            return ScriptElementKind::LocalFunctionElement;
        }
        return ScriptElementKind::FunctionElement;
    }
    if flags.contains(ast::SymbolFlags::GetAccessor) {
        return ScriptElementKind::MemberGetAccessorElement;
    }
    if flags.contains(ast::SymbolFlags::SetAccessor) {
        return ScriptElementKind::MemberSetAccessorElement;
    }
    if flags.contains(ast::SymbolFlags::Method) {
        return ScriptElementKind::MemberFunctionElement;
    }
    if flags.contains(ast::SymbolFlags::Constructor) {
        return ScriptElementKind::ConstructorImplementationElement;
    }
    if flags.contains(ast::SymbolFlags::Signature) {
        return ScriptElementKind::IndexSignatureElement;
    }

    if flags.contains(ast::SymbolFlags::Property) {
        if let Some(ch) = type_checker.as_deref_mut() {
            if flags.contains(ast::SymbolFlags::Transient)
                && symbol.check_flags.contains(ast::CheckFlags::SYNTHETIC)
            {
                let mut union_property_kind = ScriptElementKind::Unknown;
                for root_symbol in &roots {
                    if root_symbol
                        .combined_local_and_export_symbol_flags()
                        .intersects(
                            ast::SymbolFlags::PROPERTY_OR_ACCESSOR | ast::SymbolFlags::VARIABLE,
                        )
                    {
                        union_property_kind = ScriptElementKind::MemberVariableElement;
                        break;
                    }
                }
                if union_property_kind == ScriptElementKind::Unknown {
                    let type_of_union_property =
                        ch.get_type_of_symbol_at_location(symbol, location);
                    if !ch.get_call_signatures(&type_of_union_property).is_empty() {
                        return ScriptElementKind::MemberFunctionElement;
                    }
                    return ScriptElementKind::MemberVariableElement;
                }
                return union_property_kind;
            }
        }
        return ScriptElementKind::MemberVariableElement;
    }

    ScriptElementKind::Unknown
}

pub fn is_first_declaration_of_symbol_parameter(symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("is_first_declaration_of_symbol_parameter"); 
    let mut current = symbol.declarations.first().cloned();
    while let Some(n) = current {
        if ast::is_parameter_declaration(&n) {
            return true;
        }
        if matches!(
            n.kind,
            SyntaxKind::BindingElement
                | SyntaxKind::ObjectBindingPattern
                | SyntaxKind::ArrayBindingPattern
        ) {
            current = n.parent();
            continue;
        }
        return false;
    }
    false
}

pub fn is_local_variable_or_function(symbol: &Arc<Symbol>) -> bool { ::tsox_core::fntrace::enter("is_local_variable_or_function"); 
    if symbol.parent().is_some() {
        return false;
    }
    for decl in &symbol.declarations {
        if decl.kind == SyntaxKind::FunctionExpression {
            return true;
        }
        if decl.kind != SyntaxKind::VariableDeclaration
            && decl.kind != SyntaxKind::FunctionDeclaration
        {
            continue;
        }
        let mut parent = decl.parent();
        loop {
            let is_function_block = parent.as_ref().map_or(false, |p| ast::is_function_block(p));
            if is_function_block {
                break;
            }
            let Some(p) = parent.as_ref() else {
                break;
            };
            if p.kind == SyntaxKind::SourceFile || p.kind == SyntaxKind::ModuleBlock {
                break;
            }
            parent = p.parent();
        }
        if parent.as_ref().map_or(false, |p| ast::is_function_block(p)) {
            return true;
        }
    }
    false
}

pub fn is_deprecated_declaration(
    mut type_checker: Option<&mut Checker>,
    declaration: &Arc<Node>,
) -> bool { ::tsox_core::fntrace::enter("is_deprecated_declaration"); 
    match type_checker {
        Some(ch) => ch.is_deprecated_declaration(declaration),
        None => ast::mig::m3f_4::is_deprecated_declaration(declaration),
    }
}

pub fn get_normalized_symbol_modifiers(
    mut type_checker: Option<&mut Checker>,
    symbol: &Arc<Symbol>,
) -> ScriptElementKindModifier { ::tsox_core::fntrace::enter("get_normalized_symbol_modifiers"); 
    let mut modifier_set = ScriptElementKindModifier::NONE;
    let declarations = symbol.declarations.clone();
    if let Some(declaration) = declarations.first() {
        let rest = &declarations[1..];
        let exclude_flags = if !rest.is_empty()
            && is_deprecated_declaration(type_checker.as_deref_mut(), declaration)
            && rest
                .iter()
                .any(|d| !is_deprecated_declaration(type_checker.as_deref_mut(), d))
        {
            ast::ModifierFlags::Deprecated
        } else {
            ast::ModifierFlags::empty()
        };
        modifier_set = get_node_modifiers(type_checker, declaration, exclude_flags);
    }
    modifier_set
}

pub fn get_node_modifiers(
    mut type_checker: Option<&mut Checker>,
    node: &Arc<Node>,
    exclude_flags: ast::ModifierFlags,
) -> ScriptElementKindModifier { ::tsox_core::fntrace::enter("get_node_modifiers"); 
    let mut result = ScriptElementKindModifier::NONE;
    let mut flags = ast::ModifierFlags::empty();
    if ast::is_declaration(node) {
        flags = ast::get_combined_modifier_flags(node);
        if is_deprecated_declaration(type_checker.as_deref_mut(), node) {
            flags |= ast::ModifierFlags::Deprecated;
        }
        flags &= !exclude_flags;
    }

    if flags.contains(ast::ModifierFlags::Private) {
        result |= ScriptElementKindModifier::PRIVATE;
    }
    if flags.contains(ast::ModifierFlags::Protected) {
        result |= ScriptElementKindModifier::PROTECTED;
    }
    if flags.contains(ast::ModifierFlags::Public) {
        result |= ScriptElementKindModifier::PUBLIC;
    }
    if flags.contains(ast::ModifierFlags::Static) {
        result |= ScriptElementKindModifier::STATIC;
    }
    if flags.contains(ast::ModifierFlags::Abstract) {
        result |= ScriptElementKindModifier::ABSTRACT;
    }
    if flags.contains(ast::ModifierFlags::Export) {
        result |= ScriptElementKindModifier::EXPORTED;
    }
    if flags.contains(ast::ModifierFlags::Deprecated) {
        result |= ScriptElementKindModifier::DEPRECATED;
    }
    if flags.contains(ast::ModifierFlags::Ambient) {
        result |= ScriptElementKindModifier::AMBIENT;
    }
    if node.flags.contains(ast::NodeFlags::Ambient) {
        result |= ScriptElementKindModifier::AMBIENT;
    }
    if node.kind == SyntaxKind::ExportAssignment {
        result |= ScriptElementKindModifier::EXPORTED;
    }

    result
}
