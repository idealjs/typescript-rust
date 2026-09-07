use super::*;
use crate::parser::Parser;

fn parse_and_bind(source: &str) -> (Arc<SourceFile>, NodeSymbolMap) {
    let source_file = Arc::new(Parser::parse_source_file_text(
        "test.ts",
        source.to_string(),
    ));
    let symbol_map = bind_source_file(&Arc::clone(&source_file));
    (source_file, symbol_map)
}

#[test]
fn bind_variable_declaration() {
    let (file, map) = parse_and_bind("var x = 1;");
    let statements = match &file.node.data {
        NodeData::SourceFile(data) => &data.statements,
        _ => unreachable!(),
    };
    assert!(!statements.nodes.is_empty());

    let var_stmt = &statements.nodes[0];
    assert_eq!(var_stmt.kind, SyntaxKind::VariableStatement);

    let mut binder = Binder::new();
    binder.bind_source_file(&Arc::clone(&file));
    assert!(binder.symbol_count() >= 2);
    let _ = map;
}

#[test]
fn bind_function_declaration() {
    let (file, _map) = parse_and_bind("function foo() { return 42; }");
    let mut binder = Binder::new();
    binder.bind_source_file(&Arc::clone(&file));
    assert!(binder.symbol_count() >= 2);
}

#[test]
fn bind_class_declaration() {
    let (file, _map) = parse_and_bind("class Foo { bar() {} }");
    let mut binder = Binder::new();
    binder.bind_source_file(&Arc::clone(&file));
    assert!(binder.symbol_count() >= 3);
}

#[test]
fn bind_interface_declaration() {
    let (file, _map) = parse_and_bind("interface Foo { bar: number; }");
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.symbol_count() >= 3);
}

#[test]
fn bind_import_declaration() {
    let (file, _map) = parse_and_bind("import { foo } from 'mod';");
    let mut binder = Binder::new();
    binder.bind_source_file(&file);

    let _ = binder.symbol_count();
}

#[test]
fn bind_multiple_declarations() {
    let (file, _map) = parse_and_bind("let x = 1; let y = 2; let z = 3;");
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.symbol_count() >= 4);
}

#[test]
fn bind_nested_scope() {
    let (file, _map) = parse_and_bind("function foo() { let x = 1; }");
    let mut binder = Binder::new();
    binder.bind_source_file(&file);

    assert!(binder.symbol_count() >= 3);
}

#[test]
fn flow_start_node_exists() {
    let (file, map) = parse_and_bind("let x = 1;");

    let flow = map.flow_node_of(&file.node);
    assert!(flow.is_some());
    let flow = flow.unwrap();
    assert!(flow.flags.contains(FlowFlags::START));
}

#[test]
fn flow_identifier_has_flow_node() {
    let (file, map) = parse_and_bind("let x = 1; x;");

    let statements = match &file.node.data {
        NodeData::SourceFile(data) => &data.statements,
        _ => unreachable!(),
    };

    let expr_stmt = &statements.nodes[1];
    let expr = match &expr_stmt.data {
        NodeData::ExpressionStatement(data) => &data.expression,
        _ => unreachable!(),
    };
    assert_eq!(expr.kind, SyntaxKind::Identifier);

    assert!(map.flow_node_of(expr).is_some());
}

#[test]
fn flow_if_statement_merges() {
    let (file, _map) = parse_and_bind("let x = 1; if (x > 0) { x = 2; } else { x = 3; }");
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.symbol_count() >= 2);
}

#[test]
fn flow_while_statement() {
    let (file, _map) = parse_and_bind("let i = 0; while (i < 10) { i = i + 1; }");
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.symbol_count() >= 2);
}

#[test]
fn flow_for_statement() {
    let (file, _map) = parse_and_bind("for (let i = 0; i < 10; i++) { console.log(i); }");
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.symbol_count() >= 2);
}

#[test]
fn flow_switch_statement() {
    let (file, _map) =
        parse_and_bind("let x = 1; switch (x) { case 1: x = 2; break; default: x = 0; }");
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.symbol_count() >= 2);
}

#[test]
fn flow_return_statement_unreachable() {
    let (file, map) = parse_and_bind("function foo() { return 1; let x = 2; }");
    let _ = map;
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.has_explicit_return);
}

#[test]
fn flow_throw_statement() {
    let (file, _map) = parse_and_bind("function foo() { throw new Error(); }");
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.has_flow_effects);
}

#[test]
fn flow_assignment_has_effects() {
    let (file, _map) = parse_and_bind("let x = 1; x = 2;");
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.has_flow_effects);
}

#[test]
fn flow_call_expression_has_effects() {
    let (file, _map) = parse_and_bind("console.log('hello');");
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.has_flow_effects);
}

#[test]
fn flow_try_catch_finally_does_not_crash() {
    let (file, _map) =
        parse_and_bind("try { let x = 1; } catch (e) { let y = 2; } finally { let z = 3; }");
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.has_flow_effects);
}

#[test]
fn flow_try_with_throw_in_catch() {
    let (file, _map) = parse_and_bind(
        "function f() {\
         try { throw new Error(); }\
         catch (e) { return 1; }\
         return 2;\
         }",
    );
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.has_flow_effects);
}

#[test]
fn flow_labeled_break_to_outer_loop() {
    let (file, _map) = parse_and_bind(
        "outer: for (let i = 0; i < 3; i++) {\
         for (let j = 0; j < 3; j++) {\
         if (j === 1) break outer;\
         }\
         }",
    );
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.has_flow_effects);
}

#[test]
fn flow_labeled_continue_to_outer_loop() {
    let (file, _map) = parse_and_bind(
        "outer: for (let i = 0; i < 3; i++) {\
         for (let j = 0; j < 3; j++) {\
         if (j === 1) continue outer;\
         }\
         }",
    );
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.has_flow_effects);
}

#[test]
fn flow_array_mutation_call_has_effects() {
    let (file, _map) = parse_and_bind("let arr = []; arr.push(1);");
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    assert!(binder.has_flow_effects);
}

fn file_symbol<'a>(file: &'a SourceFile, map: &'a NodeSymbolMap) -> &'a Arc<Symbol> {
    map.symbols
        .get(&file.node.id())
        .expect("source file should have a symbol")
}

fn find_statement(file: &SourceFile, kind: SyntaxKind) -> Option<Arc<Node>> {
    let NodeData::SourceFile(data) = &file.node.data else {
        return None;
    };
    data.statements
        .nodes
        .iter()
        .find(|n| n.kind == kind)
        .cloned()
}

fn find_child(node: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> {
    let mut found: Option<Arc<Node>> = None;
    crate::ast::node_data_generated::for_each_child(node, |child| {
        if child.kind == kind {
            found = Some(Arc::clone(child));
            true
        } else {
            false
        }
    });
    found
}

fn find_descendant(node: &Arc<Node>, kind: SyntaxKind) -> Option<Arc<Node>> {
    if node.kind == kind {
        return Some(Arc::clone(node));
    }
    let mut found: Option<Arc<Node>> = None;
    crate::ast::node_data_generated::for_each_child(node, |child| {
        if found.is_none() {
            found = find_descendant(child, kind);
        }
        found.is_some()
    });
    found
}

#[test]
fn bind_export_default_expression_creates_default_export_symbol() {
    let (file, map) = parse_and_bind("export default 42;");
    let export_assignment =
        find_statement(&file, SyntaxKind::ExportAssignment).expect("export assignment");
    let sym = map.symbol_of(&export_assignment).expect("symbol");
    assert!(
        sym.flags.contains(SymbolFlags::Property),
        "expected Property flags, got {:?}",
        sym.flags
    );
    assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_DEFAULT);
    let file_sym = file_symbol(&file, &map);
    let default_export = file_sym
        .exports
        .get(INTERNAL_SYMBOL_NAME_DEFAULT)
        .expect("default export in file exports");
    assert!(Arc::ptr_eq(default_export, sym));
}

#[test]
fn bind_export_default_identifier_creates_alias() {
    let (file, map) = parse_and_bind("const foo = 1; export default foo;");
    let export_assignment =
        find_statement(&file, SyntaxKind::ExportAssignment).expect("export assignment");
    let sym = map.symbol_of(&export_assignment).expect("symbol");
    assert!(
        sym.flags.contains(SymbolFlags::Alias),
        "expected Alias flags, got {:?}",
        sym.flags
    );
    assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_DEFAULT);
}

#[test]
fn bind_export_equals_creates_export_equals_symbol() {
    let (file, map) = parse_and_bind("function x() {} export = x;");
    let export_assignment =
        find_statement(&file, SyntaxKind::ExportAssignment).expect("export assignment");
    let sym = map.symbol_of(&export_assignment).expect("symbol");
    assert!(sym.flags.contains(SymbolFlags::Alias));
    assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_EXPORT_EQUALS);
    assert!(
        sym.value_declaration.is_some(),
        "export = should have a value declaration set"
    );
    let file_sym = file_symbol(&file, &map);
    assert!(
        file_sym
            .exports
            .get(INTERNAL_SYMBOL_NAME_EXPORT_EQUALS)
            .is_some()
    );
}

#[test]
fn bind_export_star_creates_export_star_symbol() {
    let (file, map) = parse_and_bind("export * from \"mod\";");
    let export_decl =
        find_statement(&file, SyntaxKind::ExportDeclaration).expect("export declaration");
    let sym = map.symbol_of(&export_decl).expect("symbol");
    assert!(
        sym.flags.contains(SymbolFlags::ExportStar),
        "expected ExportStar flags, got {:?}",
        sym.flags
    );
    assert_eq!(sym.name, INTERNAL_SYMBOL_NAME_EXPORT_STAR);
    let file_sym = file_symbol(&file, &map);
    assert!(
        file_sym
            .exports
            .get(INTERNAL_SYMBOL_NAME_EXPORT_STAR)
            .is_some()
    );
}

#[test]
fn bind_export_star_as_ns_creates_alias() {
    let (file, map) = parse_and_bind("export * as ns from \"mod\";");
    let export_decl =
        find_statement(&file, SyntaxKind::ExportDeclaration).expect("export declaration");
    let ns_clause =
        find_child(&export_decl, SyntaxKind::NamespaceExport).expect("NamespaceExport clause");
    let sym = map
        .symbol_of(&ns_clause)
        .expect("symbol on NamespaceExport clause");
    assert!(sym.flags.contains(SymbolFlags::Alias));
    assert_eq!(sym.name, "ns");
    let file_sym = file_symbol(&file, &map);
    let ns_export = file_sym.exports.get("ns").expect("ns export");
    assert!(Arc::ptr_eq(ns_export, sym));
}

#[test]
fn bind_export_named_specifiers_does_not_duplicate() {
    let (file, map) = parse_and_bind("const a = 1; const b = 2; export { a, b };");
    let export_decl =
        find_statement(&file, SyntaxKind::ExportDeclaration).expect("export declaration");

    assert!(
        map.symbol_of(&export_decl).is_none(),
        "export {{ a, b }} should not create a symbol on the ExportDeclaration"
    );
}

#[test]
fn bind_import_clause_default_import_creates_local_alias() {
    let (file, map) = parse_and_bind("import D from \"mod\";");
    let import_decl =
        find_statement(&file, SyntaxKind::ImportDeclaration).expect("import declaration");
    let clause = find_child(&import_decl, SyntaxKind::ImportClause).expect("import clause");
    let sym = map.symbol_of(&clause).expect("symbol on ImportClause");
    assert!(sym.flags.contains(SymbolFlags::Alias));
    assert_eq!(sym.name, "D");
    let locals = map.locals.get(&file.node.id()).expect("file locals table");
    let local_sym = locals.get("D").expect("D in file locals");
    assert!(Arc::ptr_eq(local_sym, sym));
    let file_sym = file_symbol(&file, &map);
    assert!(
        file_sym.exports.get("D").is_none(),
        "default import should not be in exports"
    );
}

#[test]
fn bind_import_clause_without_name_is_noop() {
    let (file, map) = parse_and_bind("import { x } from \"mod\";");
    let import_decl =
        find_statement(&file, SyntaxKind::ImportDeclaration).expect("import declaration");
    let clause = find_child(&import_decl, SyntaxKind::ImportClause).expect("import clause");
    assert!(
        map.symbol_of(&clause).is_none(),
        "ImportClause without a name should not get a symbol"
    );
}

#[test]
fn bind_exported_namespace_member_has_export_symbol_link() {
    let (file, map) = parse_and_bind("namespace N { export const x = 1; }");

    let ns = find_statement(&file, SyntaxKind::ModuleDeclaration).expect("namespace N");
    let ns_sym = map.symbol_of(&ns).expect("namespace symbol");
    let x_export = ns_sym.exports.get("x").expect("x in N's exports");
    assert!(
        x_export.export_symbol.is_some(),
        "exported namespace member should have export_symbol set"
    );
    assert!(Arc::ptr_eq(
        x_export.export_symbol.as_ref().unwrap(),
        x_export
    ));
}

#[test]
fn bind_non_exported_namespace_member_has_no_export_symbol() {
    let (file, map) = parse_and_bind("namespace N { const x = 1; }");
    let ns = find_statement(&file, SyntaxKind::ModuleDeclaration).expect("namespace N");
    let ns_sym = map.symbol_of(&ns).expect("namespace symbol");
    assert!(
        ns_sym.exports.get("x").is_none(),
        "non-exported member should not be in exports"
    );

    let locals = map.locals.get(&ns.id()).expect("namespace locals table");
    let x_local = locals.get("x").expect("x in locals");
    assert!(
        x_local.export_symbol.is_none(),
        "non-exported member should not have export_symbol"
    );
}

#[test]
fn bind_exported_top_level_member_has_export_symbol_link() {
    let (file, map) = parse_and_bind("export const x = 1;");
    let var_stmt =
        find_statement(&file, SyntaxKind::VariableStatement).expect("variable statement");

    let decl_list =
        find_child(&var_stmt, SyntaxKind::VariableDeclarationList).expect("declaration list");
    let var_decl =
        find_child(&decl_list, SyntaxKind::VariableDeclaration).expect("variable declaration");
    let sym = map.symbol_of(&var_decl).expect("symbol for x");
    assert!(
        sym.export_symbol.is_some(),
        "exported top-level member should have export_symbol set"
    );
    assert!(Arc::ptr_eq(sym.export_symbol.as_ref().unwrap(), sym));
}

#[test]
fn bind_generic_alias_type_params_do_not_leak_into_file_members() {
    let (file, map) = parse_and_bind(
        "export type G<T> = { [P in T]: string };\nexport type T = G<\"a\">;\nexport const q = 1;",
    );
    let fsym = file_symbol(&file, &map);
    let t_in_file = fsym.members.get("T").or_else(|| fsym.exports.get("T"));
    let Some(t_sym) = t_in_file else {
        panic!("exported alias T should be reachable in the file symbol tables");
    };

    assert!(
        t_sym
            .declarations
            .iter()
            .all(|d| d.kind == SyntaxKind::TypeAliasDeclaration),
        "file-table T merged with a type parameter: flags={:?}",
        t_sym.flags
    );
    assert!(
        !t_sym.flags.intersects(SymbolFlags::TypeParameter),
        "exported alias T must not carry TypeParameter flags (got {:?})",
        t_sym.flags
    );

    let g_stmt = find_statement(&file, SyntaxKind::TypeAliasDeclaration).unwrap();
    let g_sym = map.symbol_of(&g_stmt).expect("symbol for G");
    assert!(
        g_sym.members.get("T").is_some(),
        "G's type parameter should live in the alias symbol's members"
    );
}

#[test]
fn bind_mapped_type_param_in_node_locals() {
    let (file, map) = parse_and_bind("type M<K extends string> = { [P in K]: number };");
    let fsym = file_symbol(&file, &map);
    assert!(
        fsym.members.get("P").is_none() && fsym.exports.get("P").is_none(),
        "mapped-type P must not leak into the file symbol tables"
    );
    let mapped = find_descendant(&file.node, SyntaxKind::MappedType).expect("mapped type node");
    let locals = map
        .locals
        .get(&mapped.id())
        .expect("mapped type node should have locals");
    assert!(
        locals.get("P").is_some(),
        "P should be in the mapped node's locals"
    );
}
