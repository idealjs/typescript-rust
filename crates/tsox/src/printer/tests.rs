use super::*;
use crate::ast::node_data_generated::{IdentifierData, NodeData};
use crate::ast::symbol::{NodeSymbolMap, SymbolFlags};
use crate::binder::Binder;
use crate::parser::Parser;

fn parse(source: &str) -> Arc<crate::ast::SourceFile> {
    let (file, _diags) =
        Parser::parse_source_file_text_with_diagnostics("test.ts", source.to_string());
    Arc::new(file)
}

fn parse_and_bind(source: &str) -> (Arc<crate::ast::SourceFile>, NodeSymbolMap) {
    let file = parse(source);
    let mut binder = Binder::new();
    binder.bind_source_file(&file);
    let symbol_map = std::mem::take(&mut binder.symbol_map);
    (file, symbol_map)
}

fn make_is_unique_local_name(symbol_map: Arc<NodeSymbolMap>) -> impl Fn(&str, &Node) -> bool {
    move |name: &str, node: &Node| -> bool {
        let mask = SymbolFlags::VALUE | SymbolFlags::ExportValue | SymbolFlags::Alias;

        if let Some(locals) = symbol_map.locals_of(node) {
            if let Some(sym) = locals.get(name) {
                if sym.flags & mask != SymbolFlags::empty() {
                    return false;
                }
            }
        }

        if let Some(sym) = symbol_map.symbol_of(node) {
            if let Some(member) = sym.members.get(name) {
                if member.flags & mask != SymbolFlags::empty() {
                    return false;
                }
            }
        }
        true
    }
}

fn make_identifier(text: &str) -> Arc<Node> {
    Arc::new(Node::new(
        SyntaxKind::Identifier,
        NodeData::Identifier(IdentifierData {
            text: text.to_string(),
        }),
    ))
}

#[test]
fn temp_variable_1() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_temp_variable();
    let name2 = factory.new_temp_variable();
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "_a");
    assert_eq!(g.generate_name(&name2), "_b");
}

#[test]
fn temp_variable_2() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_temp_variable_ex(AutoGenerateOptions {
        prefix: "A".to_string(),
        suffix: "B".to_string(),
        ..Default::default()
    });
    let name2 = factory.new_temp_variable_ex(AutoGenerateOptions {
        prefix: "A".to_string(),
        suffix: "B".to_string(),
        ..Default::default()
    });
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "A_aB");
    assert_eq!(g.generate_name(&name2), "A_bB");
}

#[test]
fn temp_variable_3() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_temp_variable();
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "_a");
    assert_eq!(g.generate_name(&name1), "_a");
}

#[test]
fn temp_variable_scoped() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_temp_variable();
    let name2 = factory.new_temp_variable();
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "_a");
    g.push_scope(false);
    assert_eq!(g.generate_name(&name2), "_a");
    g.pop_scope(false);
}

#[test]
fn temp_variable_scoped_reserved() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_temp_variable_ex(AutoGenerateOptions {
        flags: GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES,
        ..Default::default()
    });
    let name2 = factory.new_temp_variable();
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "_a");
    g.push_scope(false);
    assert_eq!(g.generate_name(&name2), "_b");
    g.pop_scope(false);
}

#[test]
fn loop_variable_1() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_loop_variable();
    let name2 = factory.new_loop_variable();
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "_i");
    assert_eq!(g.generate_name(&name2), "_a");
}

#[test]
fn loop_variable_2() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_loop_variable_ex(AutoGenerateOptions {
        prefix: "A".to_string(),
        suffix: "B".to_string(),
        ..Default::default()
    });
    let name2 = factory.new_loop_variable_ex(AutoGenerateOptions {
        prefix: "A".to_string(),
        suffix: "B".to_string(),
        ..Default::default()
    });
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "A_iB");
    assert_eq!(g.generate_name(&name2), "A_aB");
}

#[test]
fn loop_variable_3() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_loop_variable();
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "_i");
    assert_eq!(g.generate_name(&name1), "_i");
}

#[test]
fn loop_variable_scoped() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_loop_variable();
    let name2 = factory.new_loop_variable();
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "_i");
    g.push_scope(false);
    assert_eq!(g.generate_name(&name2), "_i");
    g.pop_scope(false);
}

#[test]
fn unique_name_1() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_unique_name("foo");
    let name2 = factory.new_unique_name("foo");
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "foo_1");
    assert_eq!(g.generate_name(&name2), "foo_2");
}

#[test]
fn unique_name_2() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_unique_name("foo");
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "foo_1");
    assert_eq!(g.generate_name(&name1), "foo_1");
}

#[test]
fn unique_name_scoped() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_unique_name("foo");
    let name2 = factory.new_unique_name("foo");
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "foo_1");
    g.push_scope(false);
    assert_eq!(g.generate_name(&name2), "foo_2");
    g.pop_scope(false);
}

#[test]
fn unique_private_name_1() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_unique_private_name("#foo");
    let name2 = factory.new_unique_private_name("#foo");
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "#foo_1");
    assert_eq!(g.generate_name(&name2), "#foo_2");
}

#[test]
fn unique_private_name_2() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_unique_private_name("#foo");
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "#foo_1");
    assert_eq!(g.generate_name(&name1), "#foo_1");
}

#[test]
fn unique_private_name_scoped() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let name1 = factory.new_unique_private_name("#foo");
    let name2 = factory.new_unique_private_name("#foo");
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "#foo_1");
    g.push_scope(false);
    assert_eq!(g.generate_name(&name2), "#foo_2");
    g.pop_scope(false);
}

#[test]
fn generated_name_for_identifier_1() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("function f() {}");
    let stmt = &file.node.data;
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = stmt else {
        panic!("expected SourceFile");
    };
    let func = &d.statements.nodes[0];
    let n = func.name().unwrap();
    let name1 = factory.new_generated_name_for_node(n);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "f_1");
}

#[test]
fn generated_name_for_identifier_2() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("function f() {}");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let func = &d.statements.nodes[0];
    let n = func.name().unwrap();
    let name1 = factory.new_generated_name_for_node_ex(
        n,
        AutoGenerateOptions {
            prefix: "a".to_string(),
            suffix: "b".to_string(),
            ..Default::default()
        },
    );
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "afb");
}

#[test]
fn generated_name_for_identifier_3() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("function f() {}");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let func = &d.statements.nodes[0];
    let n = func.name().unwrap();
    let _name1 = factory.new_generated_name_for_node_ex(
        n,
        AutoGenerateOptions {
            prefix: "a".to_string(),
            suffix: "b".to_string(),
            ..Default::default()
        },
    );

    let afb_node = make_identifier("afb");
    let name2 = factory.new_generated_name_for_node(&afb_node);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name2), "afb_1");
}

#[test]
fn generated_name_for_namespace_1() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, symbol_map) = parse_and_bind("namespace foo { }");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let ns1 = &d.statements.nodes[0];
    let name1 = factory.new_generated_name_for_node(ns1);
    let mut g = NameGenerator::new()
        .with_is_unique_local_name(make_is_unique_local_name(Arc::new(symbol_map)));
    assert_eq!(g.generate_name(&name1), "foo");
}

#[test]
fn generated_name_for_namespace_2() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, symbol_map) = parse_and_bind("namespace foo { var foo; }");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let ns1 = &d.statements.nodes[0];
    let name1 = factory.new_generated_name_for_node(ns1);
    let mut g = NameGenerator::new()
        .with_is_unique_local_name(make_is_unique_local_name(Arc::new(symbol_map)));
    assert_eq!(g.generate_name(&name1), "foo_1");
}

#[test]
fn generated_name_for_namespace_3() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, symbol_map) = parse_and_bind(
        "namespace ns1 { namespace foo { var foo; } } namespace ns2 { namespace foo { var foo; } }",
    );
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let ns1_outer = &d.statements.nodes[0];
    let ns2_outer = &d.statements.nodes[1];
    let crate::ast::node_data_generated::NodeData::ModuleDeclaration(ns1_data) = &ns1_outer.data
    else {
        panic!("expected ModuleDeclaration");
    };
    let ns1_body = ns1_data.body.as_ref().unwrap();
    let inner_ns1 = get_module_block_statements(ns1_body).unwrap()[0].clone();
    let name1 = factory.new_generated_name_for_node(&inner_ns1);

    let crate::ast::node_data_generated::NodeData::ModuleDeclaration(ns2_data) = &ns2_outer.data
    else {
        panic!("expected ModuleDeclaration");
    };
    let ns2_body = ns2_data.body.as_ref().unwrap();
    let inner_ns2 = get_module_block_statements(ns2_body).unwrap()[0].clone();
    let name2 = factory.new_generated_name_for_node(&inner_ns2);

    let mut g = NameGenerator::new()
        .with_is_unique_local_name(make_is_unique_local_name(Arc::new(symbol_map)));
    assert_eq!(g.generate_name(&name1), "foo_1");
    assert_eq!(g.generate_name(&name2), "foo_2");
}

#[test]
fn generated_name_for_namespace_4() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, symbol_map) = parse_and_bind(
        "namespace ns1 { namespace foo { var foo; } } namespace ns2 { namespace foo { var foo; } }",
    );
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let ns1_outer = &d.statements.nodes[0];
    let ns2_outer = &d.statements.nodes[1];
    let crate::ast::node_data_generated::NodeData::ModuleDeclaration(ns1_data) = &ns1_outer.data
    else {
        panic!("expected ModuleDeclaration");
    };
    let ns1_body = ns1_data.body.as_ref().unwrap();
    let inner_ns1 = get_module_block_statements(ns1_body).unwrap()[0].clone();
    let name1 = factory.new_generated_name_for_node(&inner_ns1);

    let crate::ast::node_data_generated::NodeData::ModuleDeclaration(ns2_data) = &ns2_outer.data
    else {
        panic!("expected ModuleDeclaration");
    };
    let ns2_body = ns2_data.body.as_ref().unwrap();
    let inner_ns2 = get_module_block_statements(ns2_body).unwrap()[0].clone();
    let name2 = factory.new_generated_name_for_node(&inner_ns2);

    let mut g = NameGenerator::new()
        .with_is_unique_local_name(make_is_unique_local_name(Arc::new(symbol_map)));
    g.push_scope(false);
    let text1 = g.generate_name(&name1);
    g.pop_scope(false);
    g.push_scope(false);
    let text2 = g.generate_name(&name2);
    g.pop_scope(false);
    assert_eq!(text1, "foo_1");
    assert_eq!(text2, "foo_2");
}

#[test]
fn generated_name_for_node_cached() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, symbol_map) = parse_and_bind("namespace foo { var foo; }");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let ns1 = &d.statements.nodes[0];
    let name1 = factory.new_generated_name_for_node(ns1);
    let name2 = factory.new_generated_name_for_node(ns1);
    let mut g = NameGenerator::new()
        .with_is_unique_local_name(make_is_unique_local_name(Arc::new(symbol_map)));
    assert_eq!(g.generate_name(&name1), "foo_1");
    assert_eq!(g.generate_name(&name2), "foo_1");
}

#[test]
fn generated_name_for_import() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("import * as foo from 'foo'");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let n = &d.statements.nodes[0];
    let name1 = factory.new_generated_name_for_node(n);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "foo_1");
}

#[test]
fn generated_name_for_export() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("export * as foo from 'foo'");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let n = &d.statements.nodes[0];
    let name1 = factory.new_generated_name_for_node(n);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "foo_1");
}

#[test]
fn generated_name_for_function_declaration_1() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("export function f() {}");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let n = &d.statements.nodes[0];
    let name1 = factory.new_generated_name_for_node(n);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "f_1");
}

#[test]
fn generated_name_for_function_declaration_2() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("export default function () {}");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let n = &d.statements.nodes[0];
    let name1 = factory.new_generated_name_for_node(n);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "default_1");
}

#[test]
fn generated_name_for_class_declaration_1() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("export class C {}");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let n = &d.statements.nodes[0];
    let name1 = factory.new_generated_name_for_node(n);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "C_1");
}

#[test]
fn generated_name_for_class_declaration_2() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("export default class {}");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let n = &d.statements.nodes[0];
    let name1 = factory.new_generated_name_for_node(n);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "default_1");
}

#[test]
fn generated_name_for_export_assignment() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("export default 0");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let n = &d.statements.nodes[0];
    let name1 = factory.new_generated_name_for_node(n);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "default_1");
}

#[test]
fn generated_name_for_class_expression() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("(class {})");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let stmt = &d.statements.nodes[0];
    let expr = stmt.expression().unwrap();
    let inner = expr.expression().unwrap();
    let name1 = factory.new_generated_name_for_node(inner);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "class_1");
}

#[test]
fn generated_name_for_method_1() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("class C { m() {} }");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let class_node = &d.statements.nodes[0];
    let crate::ast::node_data_generated::NodeData::ClassDeclaration(class_data) = &class_node.data
    else {
        panic!("expected ClassDeclaration");
    };
    let n = &class_data.members.nodes[0];
    let name1 = factory.new_generated_name_for_node(n);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "m_1");
}

#[test]
fn generated_name_for_method_2() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("class C { 0() {} }");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let class_node = &d.statements.nodes[0];
    let crate::ast::node_data_generated::NodeData::ClassDeclaration(class_data) = &class_node.data
    else {
        panic!("expected ClassDeclaration");
    };
    let n = &class_data.members.nodes[0];
    let name1 = factory.new_generated_name_for_node(n);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "_a");
}

#[test]
fn generated_private_name_for_method() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("class C { m() {} }");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let class_node = &d.statements.nodes[0];
    let crate::ast::node_data_generated::NodeData::ClassDeclaration(class_data) = &class_node.data
    else {
        panic!("expected ClassDeclaration");
    };
    let n = &class_data.members.nodes[0];
    let name1 = factory.new_generated_private_name_for_node(n);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "#m_1");
}

#[test]
fn generated_name_for_computed_property_name() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("class C { [x] }");
    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let class_node = &d.statements.nodes[0];
    let crate::ast::node_data_generated::NodeData::ClassDeclaration(class_data) = &class_node.data
    else {
        panic!("expected ClassDeclaration");
    };
    let member = &class_data.members.nodes[0];
    let n = member.name().unwrap();
    let name1 = factory.new_generated_name_for_node(n);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "_a");
}

#[test]
fn generated_name_for_other() {
    let ec = EmitContext::new();
    let factory = NodeFactory::new(&ec);
    let (file, _) = parse_and_bind("class C { [x] }");

    let crate::ast::node_data_generated::NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    let class_node = &d.statements.nodes[0];
    let crate::ast::node_data_generated::NodeData::ClassDeclaration(class_data) = &class_node.data
    else {
        panic!("expected ClassDeclaration");
    };
    let member = &class_data.members.nodes[0];
    let name1 = factory.new_generated_name_for_node(member);
    let mut g = NameGenerator::new();
    assert_eq!(g.generate_name(&name1), "_a");
}

#[test]
fn escape_string_test() {
    let cases: &[(&str, QuoteChar, &str)] = &[
        ("", QuoteChar::DoubleQuote, ""),
        ("abc", QuoteChar::DoubleQuote, "abc"),
        ("ab\"c", QuoteChar::DoubleQuote, "ab\\\"c"),
        ("ab\tc", QuoteChar::DoubleQuote, "ab\\tc"),
        ("ab\nc", QuoteChar::DoubleQuote, "ab\\nc"),
        ("ab'c", QuoteChar::DoubleQuote, "ab'c"),
        ("ab'c", QuoteChar::SingleQuote, "ab\\'c"),
        ("ab\"c", QuoteChar::SingleQuote, "ab\"c"),
        ("ab`c", QuoteChar::Backtick, "ab\\`c"),
        ("\u{001f}", QuoteChar::Backtick, "\\u001F"),
    ];
    for (i, (s, qc, expected)) in cases.iter().enumerate() {
        let actual = escape_string(s, *qc);
        assert_eq!(actual, *expected, "[{i}] escape_string({s:?}, {qc:?})");
    }
}

#[test]
fn escape_non_ascii_string_test() {
    let cases: &[(&str, QuoteChar, &str)] = &[
        ("", QuoteChar::DoubleQuote, ""),
        ("abc", QuoteChar::DoubleQuote, "abc"),
        ("ab\"c", QuoteChar::DoubleQuote, "ab\\\"c"),
        ("ab\tc", QuoteChar::DoubleQuote, "ab\\tc"),
        ("ab\nc", QuoteChar::DoubleQuote, "ab\\nc"),
        ("ab'c", QuoteChar::DoubleQuote, "ab'c"),
        ("ab'c", QuoteChar::SingleQuote, "ab\\'c"),
        ("ab\"c", QuoteChar::SingleQuote, "ab\"c"),
        ("ab`c", QuoteChar::Backtick, "ab\\`c"),
        ("ab\u{008f}c", QuoteChar::DoubleQuote, "ab\\u008Fc"),
        (
            "\u{1D7D8}\u{1D7D9}",
            QuoteChar::DoubleQuote,
            "\\uD835\\uDFD8\\uD835\\uDFD9",
        ),
    ];
    for (i, (s, qc, expected)) in cases.iter().enumerate() {
        let actual = escape_non_ascii_string(s, *qc);
        assert_eq!(
            actual, *expected,
            "[{i}] escape_non_ascii_string({s:?}, {qc:?})"
        );
    }
}

#[test]
fn escape_jsx_attribute_string_test() {
    let cases: &[(&str, QuoteChar, &str)] = &[
        ("", QuoteChar::DoubleQuote, ""),
        ("abc", QuoteChar::DoubleQuote, "abc"),
        ("ab\"c", QuoteChar::DoubleQuote, "ab&quot;c"),
        ("ab\tc", QuoteChar::DoubleQuote, "ab&#x9;c"),
        ("ab\nc", QuoteChar::DoubleQuote, "ab&#xA;c"),
        ("ab'c", QuoteChar::DoubleQuote, "ab'c"),
        ("ab'c", QuoteChar::SingleQuote, "ab&apos;c"),
        ("ab\"c", QuoteChar::SingleQuote, "ab\"c"),
        ("ab\u{008f}c", QuoteChar::DoubleQuote, "ab\u{008f}c"),
        (
            "\u{1D7D8}\u{1D7D9}",
            QuoteChar::DoubleQuote,
            "\u{1D7D8}\u{1D7D9}",
        ),
    ];
    for (i, (s, qc, expected)) in cases.iter().enumerate() {
        let actual = escape_jsx_attribute_string(s, *qc);
        assert_eq!(
            actual, *expected,
            "[{i}] escape_jsx_attribute_string({s:?}, {qc:?})"
        );
    }
}

#[test]
fn is_recognized_triple_slash_comment_test() {
    struct TsCase {
        text: &'static str,
        explicit: Option<(CommentRangeKind, usize, usize)>,
        expected: bool,
    }

    let cases: &[TsCase] = &[
        TsCase {
            text: "",
            explicit: Some((CommentRangeKind::MultiLine, 0, 0)),
            expected: false,
        },
        TsCase {
            text: "",
            explicit: Some((CommentRangeKind::SingleLine, 0, 0)),
            expected: false,
        },
        TsCase {
            text: "/a",
            explicit: None,
            expected: false,
        },
        TsCase {
            text: "//",
            explicit: None,
            expected: false,
        },
        TsCase {
            text: "//a",
            explicit: None,
            expected: false,
        },
        TsCase {
            text: "///",
            explicit: None,
            expected: false,
        },
        TsCase {
            text: "///a",
            explicit: None,
            expected: false,
        },
        TsCase {
            text: r#"///<reference path="foo" />"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"///<reference types="foo" />"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"///<reference lib="foo" />"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"///<reference no-default-lib="foo" />"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"///<amd-dependency path="foo" />"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: "///<amd-module />",
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <reference path="foo" />"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <reference types="foo" />"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <reference lib="foo" />"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <reference no-default-lib="foo" />"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <amd-dependency path="foo" />"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: "/// <amd-module />",
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <reference path="foo"/>"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <reference types="foo"/>"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <reference lib="foo"/>"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <reference no-default-lib="foo"/>"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <amd-dependency path="foo"/>"#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: "/// <amd-module/>",
            explicit: None,
            expected: true,
        },
        TsCase {
            text: "/// <reference path='foo' />",
            explicit: None,
            expected: true,
        },
        TsCase {
            text: "/// <reference types='foo' />",
            explicit: None,
            expected: true,
        },
        TsCase {
            text: "/// <reference lib='foo' />",
            explicit: None,
            expected: true,
        },
        TsCase {
            text: "/// <reference no-default-lib='foo' />",
            explicit: None,
            expected: true,
        },
        TsCase {
            text: "/// <amd-dependency path='foo' />",
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <reference path="foo" />  "#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <reference types="foo" />  "#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <reference lib="foo" />  "#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <reference no-default-lib="foo" />  "#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: r#"/// <amd-dependency path="foo" />  "#,
            explicit: None,
            expected: true,
        },
        TsCase {
            text: "/// <amd-module />  ",
            explicit: None,
            expected: true,
        },
        TsCase {
            text: "/// <foo />",
            explicit: None,
            expected: false,
        },
        TsCase {
            text: "/// <reference />",
            explicit: None,
            expected: false,
        },
        TsCase {
            text: "/// <amd-dependency />",
            explicit: None,
            expected: false,
        },
    ];

    for (i, case) in cases.iter().enumerate() {
        let range = if let Some((kind, pos, end)) = case.explicit {
            CommentRange {
                kind,
                pos,
                end,
                has_trailing_new_line: false,
            }
        } else {
            CommentRange {
                kind: CommentRangeKind::SingleLine,
                pos: 0,
                end: case.text.len(),
                has_trailing_new_line: false,
            }
        };
        let actual = is_recognized_triple_slash_comment(case.text, &range);
        assert_eq!(
            actual, case.expected,
            "[{i}] is_recognized_triple_slash_comment({:?})",
            case.text
        );
    }
}

fn source_file_statements(file: &crate::ast::SourceFile) -> &[Arc<Node>] {
    let NodeData::SourceFile(d) = &file.node.data else {
        panic!("expected SourceFile");
    };
    &d.statements.nodes
}

fn first_statement(source: &str) -> Arc<Node> {
    let file = parse(source);
    let stmts = source_file_statements(&file);
    assert!(
        !stmts.is_empty(),
        "expected at least one statement: {source:?}"
    );
    stmts[0].clone()
}

fn first_expression(source: &str) -> Arc<Node> {
    let stmt = first_statement(source);
    stmt.expression()
        .unwrap_or_else(|| panic!("expected an expression: {source:?}"))
        .clone()
}

fn first_type_alias_type(source: &str) -> Arc<Node> {
    let stmt = first_statement(source);
    let NodeData::TypeAliasDeclaration(d) = &stmt.data else {
        panic!("expected TypeAliasDeclaration: {source:?}");
    };
    d.type_node.clone()
}

fn cond_parts(node: Arc<Node>) -> (Arc<Node>, Arc<Node>, Arc<Node>) {
    let NodeData::ConditionalExpression(d) = &node.data else {
        panic!("expected ConditionalExpression, got {:?}", node.kind);
    };
    (
        d.condition.clone(),
        d.when_true.clone(),
        d.when_false.clone(),
    )
}

fn cond_type_parts(node: &Node) -> (&Arc<Node>, &Arc<Node>) {
    let NodeData::ConditionalTypeNode(d) = &node.data else {
        panic!("expected ConditionalTypeNode, got {:?}", node.kind);
    };
    (&d.check_type, &d.extends_type)
}

fn binary_operator(node: &Node) -> SyntaxKind {
    let NodeData::BinaryExpression(d) = &node.data else {
        panic!("expected BinaryExpression, got {:?}", node.kind);
    };
    d.operator_token.kind
}

fn binary_left(node: &Node) -> &Arc<Node> {
    let NodeData::BinaryExpression(d) = &node.data else {
        panic!("expected BinaryExpression, got {:?}", node.kind);
    };
    &d.left
}

fn binary_right(node: &Node) -> &Arc<Node> {
    let NodeData::BinaryExpression(d) = &node.data else {
        panic!("expected BinaryExpression, got {:?}", node.kind);
    };
    &d.right
}

fn type_list(node: &Node) -> &[Arc<Node>] {
    match &node.data {
        NodeData::UnionTypeNode(d) => &d.types.nodes,
        NodeData::IntersectionTypeNode(d) => &d.types.nodes,
        _ => panic!("expected union/intersection type, got {:?}", node.kind),
    }
}

fn type_operator(node: &Node) -> SyntaxKind {
    let NodeData::TypeOperatorNode(d) = &node.data else {
        panic!("expected TypeOperatorNode, got {:?}", node.kind);
    };
    d.operator
}

fn fn_body_first_expression(stmt: &Arc<Node>) -> Arc<Node> {
    let NodeData::FunctionDeclaration(fd) = &stmt.data else {
        panic!("expected FunctionDeclaration, got {:?}", stmt.kind);
    };
    let NodeData::Block(bd) = &fd.body.as_ref().unwrap().data else {
        panic!("expected Block body");
    };
    bd.statements.nodes[0].expression().unwrap().clone()
}

#[test]
fn emit() {
    assert_eq!(
        first_expression(r#""test""#).kind,
        SyntaxKind::StringLiteral
    );
    assert_eq!(first_expression("0").kind, SyntaxKind::NumericLiteral);
    assert_eq!(first_expression("10_000").kind, SyntaxKind::NumericLiteral);
    assert_eq!(first_expression("0n").kind, SyntaxKind::BigIntLiteral);
    assert_eq!(
        first_expression("a.b").kind,
        SyntaxKind::PropertyAccessExpression
    );
    assert_eq!(
        first_expression("a?.b").kind,
        SyntaxKind::PropertyAccessExpression
    );
    assert_eq!(
        first_expression("a[b]").kind,
        SyntaxKind::ElementAccessExpression
    );
    assert_eq!(first_expression("a()").kind, SyntaxKind::CallExpression);
    assert_eq!(first_expression("new a").kind, SyntaxKind::NewExpression);
    assert_eq!(
        first_expression("(function(){})").kind,
        SyntaxKind::ParenthesizedExpression
    );
    assert_eq!(first_expression("a=>{}").kind, SyntaxKind::ArrowFunction);
    assert_eq!(first_expression("a,b").kind, SyntaxKind::BinaryExpression);
    assert_eq!(
        first_expression("a?b:c").kind,
        SyntaxKind::ConditionalExpression
    );
    assert_eq!(first_statement("{}").kind, SyntaxKind::Block);
    assert_eq!(first_statement("if(a);").kind, SyntaxKind::IfStatement);
    assert_eq!(
        first_statement("class a {}").kind,
        SyntaxKind::ClassDeclaration
    );
    assert_eq!(
        first_statement("interface a {}").kind,
        SyntaxKind::InterfaceDeclaration
    );
    assert_eq!(
        first_statement("type T = a | b").kind,
        SyntaxKind::TypeAliasDeclaration
    );
    assert_eq!(
        first_statement("enum a{b=c}").kind,
        SyntaxKind::EnumDeclaration
    );
}

#[test]
fn parenthesize_decorator() {
    let stmt = first_statement("@(a + b) class C {}");
    assert_eq!(stmt.kind, SyntaxKind::ClassDeclaration);
    let NodeData::ClassDeclaration(cd) = &stmt.data else {
        panic!("expected ClassDeclaration");
    };
    let mods = cd.modifiers.as_ref().expect("modifiers with decorator");
    let decorator = mods
        .iter()
        .find(|n| n.kind == SyntaxKind::Decorator)
        .expect("a decorator");
    let dec_expr = decorator.expression().unwrap();
    assert_eq!(dec_expr.kind, SyntaxKind::ParenthesizedExpression);
    assert_eq!(
        dec_expr.expression().unwrap().kind,
        SyntaxKind::BinaryExpression
    );
}

#[test]
fn parenthesize_computed_property_name() {
    let stmt = first_statement("class C { [(a, b)]: any; }");
    let NodeData::ClassDeclaration(cd) = &stmt.data else {
        panic!("expected ClassDeclaration");
    };
    let member = &cd.members.nodes[0];
    let name = member.name().unwrap();
    assert_eq!(name.kind, SyntaxKind::ComputedPropertyName);
    assert_eq!(
        name.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_array_literal() {
    let expr = first_expression("[(a, b)]");
    let NodeData::ArrayLiteralExpression(d) = &expr.data else {
        panic!("expected ArrayLiteralExpression");
    };
    assert_eq!(d.elements.nodes.len(), 1);
    assert_eq!(
        d.elements.nodes[0].kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_property_access_1() {
    let expr = first_expression("(a, b).c");
    assert_eq!(expr.kind, SyntaxKind::PropertyAccessExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_property_access_2() {
    let expr = first_expression("(a?.b).c");
    assert_eq!(expr.kind, SyntaxKind::PropertyAccessExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_property_access_3() {
    let expr = first_expression("(new a).b");
    assert_eq!(expr.kind, SyntaxKind::PropertyAccessExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_element_access_1() {
    let expr = first_expression("(a, b)[c]");
    assert_eq!(expr.kind, SyntaxKind::ElementAccessExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_element_access_2() {
    let expr = first_expression("(a?.b)[c]");
    assert_eq!(expr.kind, SyntaxKind::ElementAccessExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_element_access_3() {
    let expr = first_expression("(new a)[b]");
    assert_eq!(expr.kind, SyntaxKind::ElementAccessExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_call_1() {
    let expr = first_expression("(a, b)()");
    assert_eq!(expr.kind, SyntaxKind::CallExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_call_2() {
    let expr = first_expression("(a?.b)()");
    assert_eq!(expr.kind, SyntaxKind::CallExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_call_3() {
    let expr = first_expression("(new C)()");
    assert_eq!(expr.kind, SyntaxKind::CallExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_call_4() {
    let expr = first_expression("a((b, c))");
    assert_eq!(expr.kind, SyntaxKind::CallExpression);
    let NodeData::CallExpression(d) = &expr.data else {
        panic!("expected CallExpression");
    };
    assert_eq!(d.arguments.nodes.len(), 1);
    assert_eq!(
        d.arguments.nodes[0].kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_new_1() {
    let expr = first_expression("new (a, b)()");
    assert_eq!(expr.kind, SyntaxKind::NewExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_new_2() {
    let expr = first_expression("new (C())");
    assert_eq!(expr.kind, SyntaxKind::NewExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_new_3() {
    let expr = first_expression("new C((a, b))");
    assert_eq!(expr.kind, SyntaxKind::NewExpression);
    let NodeData::NewExpression(d) = &expr.data else {
        panic!("expected NewExpression");
    };
    assert_eq!(
        d.arguments.as_ref().unwrap().nodes[0].kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_tagged_template_1() {
    let expr = first_expression("(a, b) ``");
    assert_eq!(expr.kind, SyntaxKind::ParenthesizedExpression);
}

#[test]
fn parenthesize_tagged_template_2() {
    let expr = first_expression("(a?.b) ``");
    assert_eq!(expr.kind, SyntaxKind::ParenthesizedExpression);
}

#[test]
fn parenthesize_type_assertion_1() {
    let expr = first_expression("<T>(a + b)");
    assert_eq!(expr.kind, SyntaxKind::TypeAssertionExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_arrow_function_1() {
    let expr = first_expression("() => ({})");
    assert_eq!(expr.kind, SyntaxKind::ArrowFunction);
    let NodeData::ArrowFunction(d) = &expr.data else {
        panic!("expected ArrowFunction");
    };
    assert_eq!(d.body.kind, SyntaxKind::ParenthesizedExpression);
    assert_eq!(
        d.body.expression().unwrap().kind,
        SyntaxKind::ObjectLiteralExpression
    );
}

#[test]
fn parenthesize_arrow_function_2() {
    let expr = first_expression("() => ({}.a)");
    assert_eq!(expr.kind, SyntaxKind::ArrowFunction);
    let NodeData::ArrowFunction(d) = &expr.data else {
        panic!("expected ArrowFunction");
    };
    assert_eq!(d.body.kind, SyntaxKind::ParenthesizedExpression);
    assert_eq!(
        d.body.expression().unwrap().kind,
        SyntaxKind::PropertyAccessExpression
    );
}

#[test]
fn parenthesize_delete() {
    let expr = first_expression("delete (a + b)");
    assert_eq!(expr.kind, SyntaxKind::DeleteExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_void() {
    let expr = first_expression("void (a + b)");
    assert_eq!(expr.kind, SyntaxKind::VoidExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_typeof() {
    let expr = first_expression("typeof (a + b)");
    assert_eq!(expr.kind, SyntaxKind::TypeOfExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_await() {
    let expr = fn_body_first_expression(&first_statement("async function f() { await (a + b); }"));
    assert_eq!(expr.kind, SyntaxKind::AwaitExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_binary() {
    let e = first_expression("a + b * c");
    assert_eq!(binary_operator(&e), SyntaxKind::PlusToken);
    assert_eq!(binary_right(&e).kind, SyntaxKind::BinaryExpression);
    assert_eq!(binary_operator(binary_right(&e)), SyntaxKind::AsteriskToken);

    let e = first_expression("a * b + c");
    assert_eq!(binary_operator(&e), SyntaxKind::PlusToken);
    assert_eq!(binary_left(&e).kind, SyntaxKind::BinaryExpression);

    let e = first_expression("a || b && c");
    assert_eq!(binary_operator(&e), SyntaxKind::BarBarToken);
    assert_eq!(binary_right(&e).kind, SyntaxKind::BinaryExpression);

    let e = first_expression("a ** b ** c");
    assert_eq!(binary_operator(&e), SyntaxKind::AsteriskAsteriskToken);
    assert!(
        binary_left(&e).kind == SyntaxKind::BinaryExpression
            || binary_right(&e).kind == SyntaxKind::BinaryExpression
    );

    let e = first_expression("(a + b) * c");
    assert_eq!(binary_operator(&e), SyntaxKind::AsteriskToken);
    assert_eq!(binary_left(&e).kind, SyntaxKind::ParenthesizedExpression);

    let e = first_expression("a + b + c");
    assert_eq!(binary_operator(&e), SyntaxKind::PlusToken);
    assert_eq!(binary_left(&e).kind, SyntaxKind::BinaryExpression);
}

#[test]
fn parenthesize_conditional_1() {
    let (c, _, _) = cond_parts(first_expression("(a, b) ? c : d"));
    assert_eq!(c.kind, SyntaxKind::ParenthesizedExpression);
}

#[test]
fn parenthesize_conditional_2() {
    let (c, _, _) = cond_parts(first_expression("(a = b) ? c : d"));
    assert_eq!(c.kind, SyntaxKind::ParenthesizedExpression);
}

#[test]
fn parenthesize_conditional_3() {
    let (c, _, _) = cond_parts(first_expression("(() => {}) ? a : b"));
    assert_eq!(c.kind, SyntaxKind::ParenthesizedExpression);
}

#[test]
fn parenthesize_conditional_4() {
    let expr = fn_body_first_expression(&first_statement("function* g() { (yield) ? a : b; }"));
    let (c, _, _) = cond_parts(expr);
    assert_eq!(c.kind, SyntaxKind::ParenthesizedExpression);
}

#[test]
fn parenthesize_conditional_5() {
    let (_, t, _) = cond_parts(first_expression("a ? (b, c) : d"));
    assert_eq!(t.kind, SyntaxKind::ParenthesizedExpression);
}

#[test]
fn parenthesize_conditional_6() {
    let (_, _, f) = cond_parts(first_expression("a ? b : (c, d)"));
    assert_eq!(f.kind, SyntaxKind::ParenthesizedExpression);
}

#[test]
fn parenthesize_yield_1() {
    let expr = fn_body_first_expression(&first_statement("function* g() { yield (a, b); }"));
    assert_eq!(expr.kind, SyntaxKind::YieldExpression);
    let NodeData::YieldExpression(d) = &expr.data else {
        panic!("expected YieldExpression");
    };
    assert_eq!(
        d.expression.as_ref().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_spread_element_1() {
    let expr = first_expression("[...(a, b)]");
    let NodeData::ArrayLiteralExpression(d) = &expr.data else {
        panic!("expected ArrayLiteralExpression");
    };
    assert_eq!(d.elements.nodes[0].kind, SyntaxKind::SpreadElement);
    assert_eq!(
        d.elements.nodes[0].expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_spread_element_2() {
    let expr = first_expression("a(...(b, c))");
    let NodeData::CallExpression(d) = &expr.data else {
        panic!("expected CallExpression");
    };
    assert_eq!(d.arguments.nodes[0].kind, SyntaxKind::SpreadElement);
    assert_eq!(
        d.arguments.nodes[0].expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_spread_element_3() {
    let expr = first_expression("new a(...(b, c))");
    let NodeData::NewExpression(d) = &expr.data else {
        panic!("expected NewExpression");
    };
    assert_eq!(
        d.arguments.as_ref().unwrap().nodes[0].kind,
        SyntaxKind::SpreadElement
    );
}

#[test]
fn parenthesize_expression_with_type_arguments() {
    let stmt = first_statement("class C extends (a, b)<D> {}");
    let NodeData::ClassDeclaration(cd) = &stmt.data else {
        panic!("expected ClassDeclaration");
    };
    let clause = &cd.heritage_clauses.as_ref().unwrap().nodes[0];
    let NodeData::HeritageClause(hd) = &clause.data else {
        panic!("expected HeritageClause");
    };
    let ewta = &hd.types.nodes[0];
    assert_eq!(ewta.kind, SyntaxKind::ExpressionWithTypeArguments);
    assert_eq!(
        ewta.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_as_expression() {
    let expr = first_expression("(a, b) as c");
    assert_eq!(expr.kind, SyntaxKind::AsExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_satisfies_expression() {
    let expr = first_expression("(a, b) satisfies c");
    assert_eq!(expr.kind, SyntaxKind::SatisfiesExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_non_null_expression() {
    let expr = first_expression("(a, b)!");
    assert_eq!(expr.kind, SyntaxKind::NonNullExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_expression_statement_1() {
    let expr = first_expression("({})");
    assert_eq!(expr.kind, SyntaxKind::ParenthesizedExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::ObjectLiteralExpression
    );
}

#[test]
fn parenthesize_expression_statement_2() {
    let expr = first_expression("(function () { })");
    assert_eq!(expr.kind, SyntaxKind::ParenthesizedExpression);
    assert_eq!(
        expr.expression().unwrap().kind,
        SyntaxKind::FunctionExpression
    );
}

#[test]
fn parenthesize_expression_statement_3() {
    let expr = first_expression("(class {})");
    assert_eq!(expr.kind, SyntaxKind::ParenthesizedExpression);
    assert_eq!(expr.expression().unwrap().kind, SyntaxKind::ClassExpression);
}

#[test]
fn parenthesize_expression_default_1() {
    let stmt = first_statement("export default (class {})");
    assert_eq!(stmt.kind, SyntaxKind::ExportAssignment);
    assert_eq!(
        stmt.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_expression_default_2() {
    let stmt = first_statement("export default (function () { })");
    assert_eq!(stmt.kind, SyntaxKind::ExportAssignment);
    assert_eq!(
        stmt.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_expression_default_3() {
    let stmt = first_statement("export default (a, b)");
    assert_eq!(stmt.kind, SyntaxKind::ExportAssignment);
    assert_eq!(
        stmt.expression().unwrap().kind,
        SyntaxKind::ParenthesizedExpression
    );
}

#[test]
fn parenthesize_array_type() {
    let t = first_type_alias_type("type _ = (a | b)[]");
    assert_eq!(t.kind, SyntaxKind::ArrayType);
    assert_eq!(t.type_node().unwrap().kind, SyntaxKind::ParenthesizedType);
}

#[test]
fn parenthesize_optional_type() {
    let t = first_type_alias_type("type _ = [(a | b)?]");
    assert_eq!(t.kind, SyntaxKind::TupleType);
    let NodeData::TupleTypeNode(td) = &t.data else {
        panic!("expected TupleTypeNode");
    };
    let elem = &td.elements.nodes[0];
    assert_eq!(elem.kind, SyntaxKind::OptionalType);
    assert_eq!(
        elem.type_node().unwrap().kind,
        SyntaxKind::ParenthesizedType
    );
}

#[test]
fn parenthesize_union_type_1() {
    let t = first_type_alias_type("type _ = a | (() => b)");
    assert_eq!(t.kind, SyntaxKind::UnionType);
    let last = type_list(&t).last().unwrap();
    assert_eq!(last.kind, SyntaxKind::ParenthesizedType);
    assert_eq!(last.type_node().unwrap().kind, SyntaxKind::FunctionType);
}

#[test]
fn parenthesize_union_type_2() {
    let t = first_type_alias_type("type _ = (infer a extends b) | c");
    assert_eq!(t.kind, SyntaxKind::UnionType);
    let first = &type_list(&t)[0];
    assert_eq!(first.kind, SyntaxKind::ParenthesizedType);
    assert_eq!(first.type_node().unwrap().kind, SyntaxKind::InferType);
}

#[test]
fn parenthesize_intersection_type() {
    let t = first_type_alias_type("type _ = a & (b | c)");
    assert_eq!(t.kind, SyntaxKind::IntersectionType);
    let last = type_list(&t).last().unwrap();
    assert_eq!(last.kind, SyntaxKind::ParenthesizedType);
    assert_eq!(last.type_node().unwrap().kind, SyntaxKind::UnionType);
}

#[test]
fn parenthesize_readonly_type_operator_1() {
    let t = first_type_alias_type("type _ = readonly (a | b)");
    assert_eq!(t.kind, SyntaxKind::TypeOperator);
    assert_eq!(type_operator(&t), SyntaxKind::ReadonlyKeyword);
    assert_eq!(t.type_node().unwrap().kind, SyntaxKind::ParenthesizedType);
}

#[test]
fn parenthesize_readonly_type_operator_2() {
    let t = first_type_alias_type("type _ = readonly (keyof a)");
    assert_eq!(t.kind, SyntaxKind::TypeOperator);
    assert_eq!(type_operator(&t), SyntaxKind::ReadonlyKeyword);
    let inner = t.type_node().unwrap();
    assert_eq!(inner.kind, SyntaxKind::ParenthesizedType);
    assert_eq!(inner.type_node().unwrap().kind, SyntaxKind::TypeOperator);
    assert_eq!(
        type_operator(inner.type_node().unwrap()),
        SyntaxKind::KeyOfKeyword
    );
}

#[test]
fn parenthesize_keyof_type_operator() {
    let t = first_type_alias_type("type _ = keyof (a | b)");
    assert_eq!(t.kind, SyntaxKind::TypeOperator);
    assert_eq!(type_operator(&t), SyntaxKind::KeyOfKeyword);
    assert_eq!(t.type_node().unwrap().kind, SyntaxKind::ParenthesizedType);
}

#[test]
fn parenthesize_indexed_access_type() {
    let t = first_type_alias_type("type _ = (a | b)[c]");
    assert_eq!(t.kind, SyntaxKind::IndexedAccessType);
    let NodeData::IndexedAccessTypeNode(d) = &t.data else {
        panic!("expected IndexedAccessTypeNode");
    };
    assert_eq!(d.object_type.kind, SyntaxKind::ParenthesizedType);
}

#[test]
fn parenthesize_conditional_type_1() {
    let t = first_type_alias_type("type _ = (() => a) extends b ? c : d");
    assert_eq!(t.kind, SyntaxKind::ConditionalType);
    let (check, _) = cond_type_parts(&t);
    assert_eq!(check.kind, SyntaxKind::ParenthesizedType);
    assert_eq!(check.type_node().unwrap().kind, SyntaxKind::FunctionType);
}

#[test]
fn parenthesize_conditional_type_2() {
    let t = first_type_alias_type("type _ = a extends (b extends c ? d : e) ? f : g");
    assert_eq!(t.kind, SyntaxKind::ConditionalType);
    let (_, ext) = cond_type_parts(&t);
    assert_eq!(ext.kind, SyntaxKind::ParenthesizedType);
    assert_eq!(ext.type_node().unwrap().kind, SyntaxKind::ConditionalType);
}

#[test]
fn parenthesize_conditional_type_3() {
    let t = first_type_alias_type("type _ = a extends () => (infer b extends c) ? d : e");
    assert_eq!(t.kind, SyntaxKind::ConditionalType);
    let (_, ext) = cond_type_parts(&t);
    assert_eq!(ext.kind, SyntaxKind::FunctionType);
    assert_eq!(ext.type_node().unwrap().kind, SyntaxKind::ParenthesizedType);
    assert_eq!(
        ext.type_node().unwrap().type_node().unwrap().kind,
        SyntaxKind::InferType
    );
}

#[test]
fn parenthesize_conditional_type_4() {
    let t = first_type_alias_type("type _ = a extends () => (infer b extends c) | d ? e : f");
    assert_eq!(t.kind, SyntaxKind::ConditionalType);
    let (_, ext) = cond_type_parts(&t);
    assert_eq!(ext.kind, SyntaxKind::FunctionType);
    let ret = ext.type_node().unwrap();
    assert_eq!(ret.kind, SyntaxKind::UnionType);
    assert_eq!(type_list(ret)[0].kind, SyntaxKind::ParenthesizedType);
    assert_eq!(
        type_list(ret)[0].type_node().unwrap().kind,
        SyntaxKind::InferType
    );
}

#[test]
fn name_generation() {
    let file = parse("var a;\nfunction f() { var a; }");
    let stmts = source_file_statements(&file);
    assert_eq!(stmts[0].kind, SyntaxKind::VariableStatement);
    assert_eq!(stmts[1].kind, SyntaxKind::FunctionDeclaration);
    let NodeData::FunctionDeclaration(fd) = &stmts[1].data else {
        panic!("expected FunctionDeclaration");
    };
    let NodeData::Block(bd) = &fd.body.as_ref().unwrap().data else {
        panic!("expected Block");
    };
    assert_eq!(bd.statements.nodes[0].kind, SyntaxKind::VariableStatement);
}

#[test]
fn no_trailing_comma_after_transform() {
    let expr = first_expression("[a!]");
    let NodeData::ArrayLiteralExpression(d) = &expr.data else {
        panic!("expected ArrayLiteralExpression");
    };
    assert_eq!(d.elements.nodes.len(), 1);
    assert_eq!(d.elements.nodes[0].kind, SyntaxKind::NonNullExpression);
    assert!(!d.elements.has_trailing_comma());
}

#[test]
fn trailing_comma_after_transform() {
    let expr = first_expression("[a!,]");
    let NodeData::ArrayLiteralExpression(d) = &expr.data else {
        panic!("expected ArrayLiteralExpression");
    };
    assert_eq!(d.elements.nodes.len(), 1);
    assert!(d.elements.has_trailing_comma());
}

#[test]
fn partially_emitted_expression() {
    let stmt =
        first_statement("function f() { return container.parent.left.expression.expression; }");
    let NodeData::FunctionDeclaration(fd) = &stmt.data else {
        panic!("expected FunctionDeclaration");
    };
    let NodeData::Block(bd) = &fd.body.as_ref().unwrap().data else {
        panic!("expected Block");
    };
    let ret = &bd.statements.nodes[0];
    assert_eq!(ret.kind, SyntaxKind::ReturnStatement);
    let NodeData::ReturnStatement(rd) = &ret.data else {
        panic!("expected ReturnStatement");
    };
    assert_eq!(
        rd.expression.as_ref().unwrap().kind,
        SyntaxKind::PropertyAccessExpression
    );
}

#[test]
fn parenthesize_binary_expression_mixing_nullish_coalescing() {
    let e = first_expression("(a ?? b) || c");
    assert_eq!(binary_operator(&e), SyntaxKind::BarBarToken);
    assert_eq!(binary_left(&e).kind, SyntaxKind::ParenthesizedExpression);

    let e = first_expression("(a ?? b) && c");
    assert_eq!(binary_operator(&e), SyntaxKind::AmpersandAmpersandToken);
    assert_eq!(binary_left(&e).kind, SyntaxKind::ParenthesizedExpression);

    let e = first_expression("a || (b ?? c)");
    assert_eq!(binary_operator(&e), SyntaxKind::BarBarToken);
    assert_eq!(binary_right(&e).kind, SyntaxKind::ParenthesizedExpression);

    let e = first_expression("a && (b ?? c)");
    assert_eq!(binary_operator(&e), SyntaxKind::AmpersandAmpersandToken);
    assert_eq!(binary_right(&e).kind, SyntaxKind::ParenthesizedExpression);

    let e = first_expression("(a || b) ?? c");
    assert_eq!(binary_operator(&e), SyntaxKind::QuestionQuestionToken);
    assert_eq!(binary_left(&e).kind, SyntaxKind::ParenthesizedExpression);

    let e = first_expression("(a && b) ?? c");
    assert_eq!(binary_operator(&e), SyntaxKind::QuestionQuestionToken);
    assert_eq!(binary_left(&e).kind, SyntaxKind::ParenthesizedExpression);

    let e = first_expression("a ?? (b || c)");
    assert_eq!(binary_operator(&e), SyntaxKind::QuestionQuestionToken);
    assert_eq!(binary_right(&e).kind, SyntaxKind::ParenthesizedExpression);

    let e = first_expression("a ?? (b && c)");
    assert_eq!(binary_operator(&e), SyntaxKind::QuestionQuestionToken);
    assert_eq!(binary_right(&e).kind, SyntaxKind::ParenthesizedExpression);
}
