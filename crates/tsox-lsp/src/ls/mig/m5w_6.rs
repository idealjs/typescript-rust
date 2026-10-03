#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashSet;
use std::sync::Arc;

use tsox_frontend::ast::{self, Node, NodeData, SourceFile, SyntaxKind};

use super::m5w_5::SourceDefResolver;

impl SourceDefResolver {
    pub fn resolver_resolve_module_name(
        &self,
        module_name: &str,
        resolve_from_file: &str,
        mode: tsox_core::core::compiler_options_kinds::ResolutionMode,
    ) -> Option<tsox_tsoptions::module::ResolvedModule> { ::tsox_core::fntrace::enter("resolver_resolve_module_name"); 
        let resolver = self.resolver_borrow();
        let (resolved, _diagnostics) =
            resolver.resolve_module_name(module_name, resolve_from_file, mode, None);
        resolved.filter(|m| m.is_resolved())
    }

    pub fn resolver_get_package_scope_for_path(
        &self,
        path: &str,
    ) -> Option<tsox_tsoptions::packagejson::mig::x12::InfoCacheEntry> { ::tsox_core::fntrace::enter("resolver_get_package_scope_for_path"); 
        let resolver = self.resolver_borrow();
        resolver.get_package_scope_for_path(path)
    }

    pub fn resolver_borrow(&self) -> std::cell::Ref<'_, tsox_tsoptions::module::Resolver> { ::tsox_core::fntrace::enter("resolver_borrow"); 
        self.resolver_cell.borrow()
    }

    pub fn fs_file_exists(&self, path: &str) -> bool { ::tsox_core::fntrace::enter("fs_file_exists"); 
        self.fs.file_exists(path)
    }

    pub fn find_declarations_in_file(
        &self,
        file_name: &str,
        names: &[String],
        seen: &mut HashSet<String>,
    ) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("find_declarations_in_file"); 
        if file_name.is_empty() || names.is_empty() {
            return Vec::new();
        }
        if !seen.insert(file_name.to_string()) {
            return Vec::new();
        }

        let Some(source_file) = self.get_or_parse_source_file(file_name) else {
            return Vec::new();
        };

        let declarations = find_declaration_nodes_by_name(&source_file, names);
        if !declarations.is_empty() && has_concrete_source_declarations(&declarations) {
            return declarations;
        }

        let mut forwarded: Vec<Arc<Node>> = Vec::new();
        for forwarded_file in self.get_forwarded_implementation_files(&source_file) {
            forwarded.extend(self.find_declarations_in_file(&forwarded_file, names, seen));
        }
        if !forwarded.is_empty() {
            if has_concrete_source_declarations(&forwarded) {
                return unique_declaration_nodes(&forwarded);
            }
            let mut combined = declarations.clone();
            combined.extend(forwarded);
            return unique_declaration_nodes(&combined);
        }
        declarations
    }

    pub fn get_forwarded_implementation_files(&self, source_file: &Arc<SourceFile>) -> Vec<String> { ::tsox_core::fntrace::enter("get_forwarded_implementation_files"); 
        let preferred_mode = self.infer_implied_node_format(&source_file.file_name);

        let mut files: Vec<String> = Vec::new();
        for imp in source_file_imports(source_file) {
            let module_name = ast::node_text(&imp);
            let implementation_file = self.resolve_implementation_from(
                &module_name,
                &source_file.file_name,
                preferred_mode,
            );
            if !implementation_file.is_empty() {
                files.push(implementation_file);
            }
        }
        crate::ls::mig::m5w_5::core_deduplicate_strings(&files)
    }
}

pub fn get_source_def_checker_info(
    program: &Arc<tsox_compile::compiler::Program>,
    file: &Arc<SourceFile>,
    node: &Arc<Node>,
) -> (Vec<Arc<Node>>, String) { ::tsox_core::fntrace::enter("get_source_def_checker_info"); 
    let mut c = program.get_type_checker_for_file(file);

    let mut declarations = crate::ls::mig::m5s::get_declarations_from_location(&mut c, node);
    let is_property_name = node
        .parent()
        .map(|parent| {
            ast::is_access_expression(&parent)
                && ast::node_name(&parent).is_some_and(|n| Arc::ptr_eq(n, node))
        })
        .unwrap_or(false);
    if declarations.is_empty() && is_property_name {
        if let Some(parent) = node.parent() {
            if let Some(left) = access_expression_expression(&parent) {
                let ty = c.get_type_at_location(&left);
                if let Some(prop) = c.get_property_of_type(&ty, &ast::node_text(node)) {
                    declarations = prop.declarations.clone();
                }
            }
        }
    }
    if let Some(called_declaration) = crate::ls::mig::m5s::try_get_signature_declaration(&mut c, node) {
        let non_function_declarations: Vec<Arc<Node>> = declarations
            .iter()
            .filter(|n| !ast::is_function_like(n))
            .cloned()
            .collect();
        declarations = non_function_declarations;
        declarations.push(called_declaration);
    }

    let mut module_specifier = String::new();
    let mut resolve_node = node.clone();
    if is_property_name {
        let mut expr = node.parent().and_then(|p| access_expression_expression(&p));
        while let Some(current) = expr.clone() {
            if !ast::is_access_expression(&current) {
                break;
            }
            expr = access_expression_expression(&current);
        }
        if let Some(expr) = expr {
            resolve_node = expr;
        }
    }
    if let Some(sym) = c.get_symbol_at_location(&resolve_node) {
        for d in &sym.declarations {
            if !ast::is_import_specifier(d)
                && !ast::is_import_clause(d)
                && !ast::is_namespace_import(d)
                && !ast::is_import_equals_declaration(d)
            {
                continue;
            }
            if let Some(spec) = tsox_checker::checker::mig::m2g::try_get_module_specifier_from_declaration_worker(d) {
                module_specifier = ast::node_text(&spec).to_string();
                break;
            }
        }
    }

    (declarations, module_specifier)
}

pub fn is_default_import_name(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_default_import_name"); 
    let Some(parent) = node.parent() else {
        return false;
    };
    if !ast::is_import_clause(&parent) || !parent.name().is_some_and(|n| Arc::ptr_eq(n, node)) {
        return false;
    }
    let Some(grand) = parent.parent() else {
        return false;
    };
    tsox_frontend::ast::mig::m3f_4::is_default_import(&grand)
}

pub fn get_source_definition_entry_node(source_file: &Arc<SourceFile>) -> Arc<Node> { ::tsox_core::fntrace::enter("get_source_definition_entry_node"); 
    if let NodeData::SourceFile(d) = &source_file.node.data {
        if !d.statements.nodes.is_empty() {
            return Arc::clone(&d.statements.nodes[0]);
        }
    }
    Arc::clone(&source_file.node)
}

pub fn get_source_definition_entry_declarations(source_file: &Arc<SourceFile>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_source_definition_entry_declarations"); 
    vec![get_source_definition_entry_node(source_file)]
}

pub fn get_candidate_source_declaration_names(
    original_node: Option<&Arc<Node>>,
    declaration: Option<&Arc<Node>>,
) -> Vec<String> { ::tsox_core::fntrace::enter("get_candidate_source_declaration_names"); 
    let mut names: Vec<String> = Vec::new();
    if let Some(declaration) = declaration {
        if let Some(name) = ast::get_name_of_declaration(declaration) {
            let text = tsox_frontend::ast::mig::w5::get_text_of_property_name(&name);
            if !text.is_empty() {
                names.push(text);
            }
        }
        if declaration.kind == SyntaxKind::ExportAssignment {
            names.push("default".to_string());
        }
        if (ast::is_function_declaration(declaration) || ast::is_class_declaration(declaration))
            && declaration
                .syntactic_modifier_flags()
                .contains(ast::ModifierFlags::ExportDefault)
        {
            names.push("default".to_string());
        }
        if ast::is_import_specifier(declaration) || ast::is_export_specifier(declaration) {
            if let Some(prop_name) = import_or_export_specifier_property_name(declaration) {
                names.push(ast::node_text(&prop_name).to_string());
            }
        }
    }
    if let Some(original_node) = original_node {
        if ast::is_identifier(original_node) || ast::is_private_identifier(original_node) {
            names.push(ast::node_text(original_node).to_string());
        }
        if is_default_import_name(original_node) {
            names.push("default".to_string());
        }
        if let Some(parent) = original_node.parent() {
            if ast::is_import_specifier(&parent) || ast::is_export_specifier(&parent) {
                if let Some(prop_name) = import_or_export_specifier_property_name(&parent) {
                    names.push(ast::node_text(&prop_name).to_string());
                }
            }
        }
    }
    names
}

pub fn find_declaration_nodes_by_name(
    source_file: &Arc<SourceFile>,
    names: &[String],
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("find_declaration_nodes_by_name"); 
    let names = crate::ls::mig::m5w_5::core_deduplicate_strings(
        &names.iter().filter(|n| !n.is_empty()).cloned().collect::<Vec<_>>(),
    );
    if names.is_empty() {
        return Vec::new();
    }

    let mut wanted: HashSet<String> = HashSet::new();
    let mut want_default = false;
    for name in &names {
        if name == "default" {
            want_default = true;
            continue;
        }
        wanted.insert(name.clone());
    }

    struct Candidate {
        node: Arc<Node>,
        depth: usize,
    }
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut min_depth = usize::MAX;

    let root = Arc::clone(&source_file.node);
    let mut stack: Vec<Arc<Node>> = Vec::new();
    tsox_frontend::ast::node_data_generated::for_each_child(&root, |child| {
        stack.push(Arc::clone(child));
        false
    });
    while let Some(node) = stack.pop() {
        let mut matched = false;
        if let Some(name) = ast::get_name_of_declaration(&node) {
            let text = tsox_frontend::ast::mig::w5::get_text_of_property_name(&name);
            if !text.is_empty() && wanted.contains(&text) {
                matched = true;
            }
        }
        if want_default && node.kind == SyntaxKind::ExportAssignment {
            matched = true;
        }
        if want_default
            && (ast::is_function_declaration(&node) || ast::is_class_declaration(&node))
            && node
                .syntactic_modifier_flags()
                .contains(ast::ModifierFlags::ExportDefault)
        {
            matched = true;
        }
        if matched {
            let depth = get_container_depth(&node);
            candidates.push(Candidate {
                node: Arc::clone(&node),
                depth,
            });
            if depth < min_depth {
                min_depth = depth;
            }
        }
        tsox_frontend::ast::node_data_generated::for_each_child(&node, |child| {
            stack.push(Arc::clone(child));
            false
        });
    }

    let declarations: Vec<Arc<Node>> = candidates
        .into_iter()
        .filter(|c| c.depth == min_depth)
        .map(|c| c.node)
        .collect();
    unique_declaration_nodes(&declarations)
}

pub fn get_container_depth(node: &Arc<Node>) -> usize { ::tsox_core::fntrace::enter("get_container_depth"); 
    let mut depth = 0usize;
    let mut current = Some(Arc::clone(node));
    while let Some(cur) = current {
        current = get_container_node(&cur);
        depth += 1;
    }
    depth
}

pub fn filter_preferred_source_declarations(
    original_node: &Arc<Node>,
    declarations: &[Arc<Node>],
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("filter_preferred_source_declarations"); 
    if declarations.len() <= 1 {
        return declarations.to_vec();
    }
    let preferred = get_property_like_source_declarations(original_node, declarations);
    if !preferred.is_empty() {
        return preferred;
    }
    let preferred: Vec<Arc<Node>> = declarations
        .iter()
        .filter(|n| is_concrete_source_declaration(n))
        .cloned()
        .collect();
    if !preferred.is_empty() {
        return preferred;
    }
    declarations.to_vec()
}

pub fn get_property_like_source_declarations(
    original_node: &Arc<Node>,
    declarations: &[Arc<Node>],
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_property_like_source_declarations"); 
    let Some(parent) = original_node.parent() else {
        return Vec::new();
    };
    if !ast::is_access_expression(&parent)
        || !ast::node_name(&parent).is_some_and(|n| Arc::ptr_eq(n, original_node))
    {
        return Vec::new();
    }
    declarations
        .iter()
        .filter(|node| {
            matches!(
                node.kind,
                SyntaxKind::PropertyAssignment
                    | SyntaxKind::ShorthandPropertyAssignment
                    | SyntaxKind::PropertyDeclaration
                    | SyntaxKind::PropertySignature
                    | SyntaxKind::MethodDeclaration
                    | SyntaxKind::MethodSignature
                    | SyntaxKind::GetAccessor
                    | SyntaxKind::SetAccessor
                    | SyntaxKind::EnumMember
            )
        })
        .cloned()
        .collect()
}

pub fn has_concrete_source_declarations(declarations: &[Arc<Node>]) -> bool { ::tsox_core::fntrace::enter("has_concrete_source_declarations"); 
    declarations.iter().any(is_concrete_source_declaration)
}

pub fn is_concrete_source_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_concrete_source_declaration"); 
    if !ast::is_declaration(node) || node.kind == SyntaxKind::ExportAssignment {
        return false;
    }
    if (ast::is_binary_expression(node) || ast::is_call_expression(node))
        && tsox_frontend::ast::mig::m3e_4::get_assignment_declaration_kind(node) != tsox_frontend::ast::mig::m3e_4::JsDeclarationKind::None
    {
        return false;
    }
    !matches!(
        node.kind,
        SyntaxKind::Parameter
            | SyntaxKind::TypeParameter
            | SyntaxKind::BindingElement
            | SyntaxKind::ImportClause
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::NamespaceImport
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ElementAccessExpression
    )
}

pub fn unique_declaration_nodes(nodes: &[Arc<Node>]) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("unique_declaration_nodes"); 
    let mut seen: HashSet<(u64, usize, usize)> = HashSet::new();
    let mut result = Vec::with_capacity(nodes.len());
    for node in nodes {
        let key = (node_root_file_id(node), node.pos(), node.end());
        if !seen.insert(key) {
            continue;
        }
        result.push(Arc::clone(node));
    }
    result
}

pub fn find_closest_declaration_node(source_file: &Arc<SourceFile>, pos: usize) -> Arc<Node> { ::tsox_core::fntrace::enter("find_closest_declaration_node"); 
    let mut current = tsox_frontend::astnav::get_touching_property_name(&source_file.node, pos);
    while let Some(cur) = current {
        if ast::is_declaration(&cur) || cur.kind == SyntaxKind::ExportAssignment {
            return cur;
        }
        current = cur.parent();
    }
    get_source_definition_entry_node(source_file)
}

pub fn find_containing_module_specifier(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("find_containing_module_specifier"); 
    let mut current = Some(node.clone());
    while let Some(cur) = current {
        if ast::is_any_import_or_re_export(&cur)
            || ast::is_require_call(&cur, true)
            || ast::is_import_call(&cur)
        {
            if let Some(module_specifier) = get_external_module_name(&cur) {
                if ast::is_string_literal_like(&module_specifier) {
                    return Some(module_specifier);
                }
            }
        }
        current = cur.parent();
    }
    None
}

pub fn get_reference_at_position(
    file: &Arc<SourceFile>,
    pos: usize,
    program: &Arc<tsox_compile::compiler::Program>,
) -> Option<M5wFileReferenceAtPosition> { ::tsox_core::fntrace::enter("get_reference_at_position"); 
    for reference in &file.referenced_files {
        if reference.range.pos as usize <= pos && pos <= reference.range.end as usize {
            let resolved = program.get_source_file_from_reference(file, reference);
            return Some(M5wFileReferenceAtPosition {
                reference: Some(Arc::new(reference.clone())),
                file: resolved,
            });
        }
    }
    None
}

pub struct M5wFileReferenceAtPosition {
    pub reference: Option<Arc<tsox_frontend::ast::node_source_file::FileReference>>,
    pub file: Option<Arc<SourceFile>>,
}

pub fn get_file_and_start_pos_from_declaration(
    program: &Arc<tsox_compile::compiler::Program>,
    declaration: &Arc<Node>,
) -> Option<(Arc<SourceFile>, usize)> { ::tsox_core::fntrace::enter("get_file_and_start_pos_from_declaration"); 
    let file = m5w_source_file_of_node(program, declaration)?;
    let start_pos = m5w_declaration_start_pos(declaration);
    Some((file, start_pos))
}

pub fn m5w_declaration_start_pos(declaration: &Arc<Node>) -> usize { ::tsox_core::fntrace::enter("m5w_declaration_start_pos"); 
    declaration.pos()
}

pub fn m5w_source_file_of_node(
    program: &Arc<tsox_compile::compiler::Program>,
    node: &Arc<Node>,
) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("m5w_source_file_of_node"); 
    let mut current: Option<Arc<Node>> = Some(Arc::clone(node));
    while let Some(cur) = current {
        if cur.kind == SyntaxKind::SourceFile {
            let root_id = cur.id();
            return program
                .get_source_files()
                .into_iter()
                .find(|f| f.node.id() == root_id);
        }
        current = cur.parent();
    }
    None
}

pub fn node_root_file_id(node: &Arc<Node>) -> u64 { ::tsox_core::fntrace::enter("node_root_file_id"); 
    let mut current = Arc::clone(node);
    while let Some(parent) = current.parent() {
        current = parent;
    }
    current.id()
}

pub fn source_file_imports(source_file: &Arc<SourceFile>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("source_file_imports"); 
    source_file.imports.clone()
}

pub fn get_container_node(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_container_node"); 
    let mut current = node.parent();
    while let Some(parent) = current {
        if matches!(
            parent.kind,
            SyntaxKind::SourceFile
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::MethodSignature
                | SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::ClassDeclaration
                | SyntaxKind::InterfaceDeclaration
                | SyntaxKind::EnumDeclaration
                | SyntaxKind::ModuleDeclaration
        ) {
            return Some(parent);
        }
        current = parent.parent();
    }
    None
}

fn access_expression_expression(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("access_expression_expression"); 
    match &node.data {
        NodeData::PropertyAccessExpression(d) => Some(d.expression.clone()),
        NodeData::ElementAccessExpression(d) => Some(d.expression.clone()),
        _ => None,
    }
}

fn import_or_export_specifier_property_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_or_export_specifier_property_name"); 
    match &node.data {
        NodeData::ImportSpecifier(d) => d.property_name.clone(),
        NodeData::ExportSpecifier(d) => d.property_name.clone(),
        _ => None,
    }
}

fn get_external_module_name(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_external_module_name"); 
    match &node.data {
        NodeData::ImportDeclaration(d) => Some(d.module_specifier.clone()),
        NodeData::ExportDeclaration(d) => d.module_specifier.clone(),
        NodeData::ImportEqualsDeclaration(d) => match &d.module_reference.data {
            NodeData::ExternalModuleReference(e) => Some(e.expression.clone()),
            _ => None,
        },
        NodeData::CallExpression(d) => {
            let arguments = d.arguments.nodes.clone();
            arguments.into_iter().next()
        }
        _ => None,
    }
}
