#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ls::find_all_references::{Definition, DefinitionKind, EntryKind, ReferenceEntry, RefOptions, ReferenceUse, SymbolAndEntries};
use crate::ls::language_service::LanguageService;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::astnav;
use tsox_frontend::scanner;

use super::m5s_3::{ImpExpKind, RefSearch, RefState};

pub fn new_node_entry(node: &Arc<Node>) -> ReferenceEntry { ::tsox_core::fntrace::enter("new_node_entry"); 
    let mut entry = ReferenceEntry {
        kind: EntryKind::Node,
        node: Some(core_or_else(node.name().cloned(), || node.clone())),
        context: None,
        file_name: String::new(),
        text_range: None,
        lsp_range: None,
    };
    entry.context = Some(super::m5s_3::get_context_node_for_node_entry(&entry.node.clone().unwrap()));
    entry
}

pub fn new_node_entry_with_kind(node: &Arc<Node>, kind: EntryKind) -> ReferenceEntry { ::tsox_core::fntrace::enter("new_node_entry_with_kind"); 
    let mut entry = new_node_entry(node);
    entry.kind = kind;
    entry
}

pub fn core_or_else<T: Clone>(value: Option<T>, fallback: impl FnOnce() -> T) -> T { ::tsox_core::fntrace::enter("core_or_else"); 
    value.unwrap_or_else(fallback)
}

pub fn get_range_of_node(node: &Arc<Node>, source_file: Option<&Arc<SourceFile>>, end_node: Option<&Arc<Node>>) -> TextRange { ::tsox_core::fntrace::enter("get_range_of_node"); 
    let Some(source_file) = source_file else {
        unimplemented!("getRangeOfNode 的 sourceFile==nil 分支依赖 Node 到 SourceFile 的回映射,全仓尚无真实实现")
    };
    let mut start = tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(node, source_file, false);
    let end_node = end_node;
    let mut end = core_or_else(end_node.map(|n| n.end()), || node.end());
    if ast::is_string_literal_like(node) && end > start + 2 {
        start += 1;
        end -= 1;
    }
    if let Some(end_node) = end_node {
        if end_node.kind == SyntaxKind::CaseBlock {
            end = end_node.pos();
        }
    }
    TextRange::new(start, end)
}

pub fn is_valid_reference_position(node: &Arc<Node>, search_symbol_name: &str) -> bool { ::tsox_core::fntrace::enter("is_valid_reference_position"); 
    match node.kind {
        SyntaxKind::PrivateIdentifier => node.text().len() == search_symbol_name.len(),
        SyntaxKind::Identifier => node.text().len() == search_symbol_name.len(),
        SyntaxKind::NoSubstitutionTemplateLiteral | SyntaxKind::StringLiteral => {
            let Some(parent) = node.parent() else {
                return false;
            };
            node.text().len() == search_symbol_name.len()
                && (super::m5x_3::is_literal_name_of_property_declaration_or_index_access(node)
                    || super::m5x_4::is_name_of_module_declaration(node)
                    || super::m5x_4::is_expression_of_external_module_import_equals_declaration(node)
                    || (parent.kind == SyntaxKind::CallExpression
                        && tsox_frontend::ast::mig::m3f_3::is_bindable_object_define_property_call(&parent)
                        && match &parent.data {
                            ast::NodeData::CallExpression(d) => {
                                d.arguments.nodes.get(1).map(|a| Arc::ptr_eq(a, node)).unwrap_or(false)
                            }
                            _ => false,
                        })
                    || ast::is_import_or_export_specifier(&parent))
        }
        SyntaxKind::NumericLiteral => {
            super::m5x_3::is_literal_name_of_property_declaration_or_index_access(node)
                && node.text().len() == search_symbol_name.len()
        }
        SyntaxKind::DefaultKeyword => "default".len() == search_symbol_name.len(),
        _ => false,
    }
}

pub fn is_for_rename_with_prefix_and_suffix_text(options: &RefOptions) -> bool { ::tsox_core::fntrace::enter("is_for_rename_with_prefix_and_suffix_text"); 
    options.use_ == ReferenceUse::Rename && options.use_aliases_for_rename
}

pub fn skip_past_export_or_import_specifier_or_union(
    symbol: &Arc<Symbol>,
    node: Option<&Arc<Node>>,
    checker: &mut tsox_checker::checker::Checker,
    use_local_symbol_for_export_specifier: bool,
) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("skip_past_export_or_import_specifier_or_union"); 
    let node = node?;
    let parent = node.parent();
    if let Some(parent) = &parent {
        if parent.kind == SyntaxKind::ExportSpecifier && use_local_symbol_for_export_specifier {
            return Some(super::m5x_5::get_local_symbol_for_export_specifier(node, symbol, parent, checker));
        }
    }
    symbol
        .declarations
        .iter()
        .find_map(|decl| {
            let parent = decl.parent()?;
            if parent.kind == SyntaxKind::TypeLiteral {
                if let Some(grandparent) = parent.parent() {
                    if grandparent.kind == SyntaxKind::UnionType {
                        let ty = checker.get_type_from_type_node(&grandparent);
                        return checker.get_property_of_type(&ty, symbol.name.as_str());
                    }
                }
            }
            None
        })
}

pub fn get_symbol_scope(symbol: &Arc<Symbol>, checker: &tsox_checker::checker::Checker) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_symbol_scope"); 
    let value_declaration = symbol.value_declaration.clone();
    if let Some(value_declaration) = value_declaration {
        if value_declaration.kind == SyntaxKind::FunctionExpression || value_declaration.kind == SyntaxKind::ClassExpression {
            return Some(value_declaration);
        }
    }
    let declarations = &symbol.declarations;
    if declarations.is_empty() {
        return None;
    }
    if symbol.flags.intersects(ast::SymbolFlags::Property | ast::SymbolFlags::Method) {
        let private_declaration = declarations.iter().find(|d| {
            tsox_frontend::ast::mig::m3f_2::has_modifier(d, ast::ModifierFlags::Private)
                || tsox_frontend::ast::mig::m3g_2::is_private_identifier_class_element_declaration(d)
        });
        if let Some(private_declaration) = private_declaration {
            return ast::find_ancestor_kind(private_declaration, SyntaxKind::ClassDeclaration);
        }
        return None;
    }
    if declarations.iter().any(super::m5x_3::is_object_binding_element_without_property_name) {
        return None;
    }
    let exposed_by_parent = symbol.parent().is_some() && !symbol.flags.contains(ast::SymbolFlags::TypeParameter);
    if exposed_by_parent {
        let parent = symbol.parent().unwrap();
        let external = tsox_checker::checker::is_external_module_symbol(&parent)
            && !parent
                .value_declaration
                .as_ref()
                .map(super::m5x_6::is_source_file_with_global_exports)
                .unwrap_or(false);
        if !external {
            return None;
        }
    }
    let mut scope: Option<Arc<Node>> = None;
    for declaration in declarations {
        let container = crate::ls::utilities::get_container_node(declaration);
        if let Some(prev) = &scope {
            match &container {
                Some(c) if !Arc::ptr_eq(prev, c) => return None,
                None => return None,
                _ => {}
            }
        }
        match &container {
            None => return None,
            Some(c) if c.kind == SyntaxKind::SourceFile && !is_external_or_commonjs_module_source_file(c) => {
                return None;
            }
            _ => {}
        }
        scope = container;
    }
    if exposed_by_parent {
        let scope = scope?;
        return ast::get_source_file_of_node(&scope);
    }
    scope
}

pub fn is_definition_visible(checker: &mut tsox_checker::checker::Checker, declaration: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_definition_visible"); 
    if checker.is_declaration_visible(declaration) {
        return true;
    }
    let Some(parent) = declaration.parent() else {
        return false;
    };
    if tsox_frontend::ast::mig::m3f_2::has_initializer(&parent)
        && tsox_frontend::ast::mig::m3f_2::node_initializer(&parent)
            .map(|i| Arc::ptr_eq(i, declaration))
            .unwrap_or(false)
    {
        return is_definition_visible(checker, &parent);
    }
    match declaration.kind {
        SyntaxKind::PropertyDeclaration | SyntaxKind::GetAccessor | SyntaxKind::SetAccessor | SyntaxKind::MethodDeclaration => {
            if tsox_frontend::ast::mig::m3f_2::has_modifier(declaration, ast::ModifierFlags::Private)
                || declaration.name().map(|n| ast::is_private_identifier(&n)).unwrap_or(false)
            {
                return false;
            }
            is_definition_visible(checker, &parent)
        }
        SyntaxKind::Constructor
        | SyntaxKind::PropertyAssignment
        | SyntaxKind::ShorthandPropertyAssignment
        | SyntaxKind::ObjectLiteralExpression
        | SyntaxKind::ClassExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::FunctionExpression => is_definition_visible(checker, &parent),
        _ => false,
    }
}

pub fn is_declaration_of_symbol(node: Option<&Arc<Node>>, target: &Arc<Symbol>, program: &tsox_compile::compiler::Program) -> bool { ::tsox_core::fntrace::enter("is_declaration_of_symbol"); 
    let Some(node) = node else {
        return false;
    };
    let symbols = program.symbol_map();
    let source = if let Some(decl) = tsox_frontend::ast::mig::m3b::get_declaration_from_name(node, symbols) {
        Some(decl)
    } else if node.kind == SyntaxKind::DefaultKeyword {
        node.parent()
    } else if tsox_frontend::ast::mig::m3g::is_literal_computed_property_declaration_name(node) {
        node.parent().and_then(|p| p.parent())
    } else if node.kind == SyntaxKind::ConstructorKeyword
        && node.parent().map(|p| ast::is_constructor_declaration(&p)).unwrap_or(false)
    {
        node.parent().and_then(|p| p.parent())
    } else {
        None
    };
    match source {
        Some(source) => target.declarations.iter().any(|decl| Arc::ptr_eq(decl, &source)),
        None => false,
    }
}

fn is_external_or_commonjs_module_source_file(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_external_or_commonjs_module_source_file"); 
    match super::m5u::node_as_source_file(node) {
        Some(file) => file.external_module_indicator.is_some() || file.common_js_module_indicator.is_some(),
        None => false,
    }
}
