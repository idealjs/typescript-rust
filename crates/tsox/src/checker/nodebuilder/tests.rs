use super::*;
use crate::bundled::lib_path;
use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
use crate::tsoptions::parse_command_line;
use crate::vfs::InMemoryFS;

fn build_checker(source: &str) -> Checker {
    let fs = Arc::new(InMemoryFS::new());
    fs.insert_dir("/proj");
    fs.insert_file("/proj/entry.ts", source);
    let args = vec!["--noLib".to_string(), "/proj/entry.ts".to_string()];
    let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));
    let host: Arc<dyn CompilerHost> =
        Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
    let program = Arc::new(Program::new(ProgramOptions {
        config: parsed,
        host,
    }));
    program.build_checker()
}

fn first_var_type_node(checker: &Checker) -> Arc<Node> {
    let file = checker
        .files
        .iter()
        .find(|f| f.file_name == "/proj/entry.ts")
        .expect("entry source file");
    let NodeData::SourceFile(sf) = &file.node.data else {
        panic!("not a source file");
    };
    for stmt in sf.statements.nodes.iter() {
        if stmt.kind != SyntaxKind::VariableStatement {
            continue;
        }
        let NodeData::VariableStatement(vs) = &stmt.data else {
            continue;
        };
        let NodeData::VariableDeclarationList(vdl) = &vs.declaration_list.data else {
            continue;
        };
        for decl in vdl.declarations.nodes.iter() {
            let NodeData::VariableDeclaration(vd) = &decl.data else {
                continue;
            };
            if let Some(tn) = &vd.type_node {
                return Arc::clone(tn);
            }
        }
    }
    panic!("no variable declaration with type annotation found");
}

fn type_node_to_string(node: &Arc<Node>) -> String {
    match node.kind {
        SyntaxKind::AnyKeyword => "any".into(),
        SyntaxKind::UnknownKeyword => "unknown".into(),
        SyntaxKind::StringKeyword => "string".into(),
        SyntaxKind::NumberKeyword => "number".into(),
        SyntaxKind::BigIntKeyword => "bigint".into(),
        SyntaxKind::BooleanKeyword => "boolean".into(),
        SyntaxKind::SymbolKeyword => "symbol".into(),
        SyntaxKind::VoidKeyword => "void".into(),
        SyntaxKind::UndefinedKeyword => "undefined".into(),
        SyntaxKind::NullKeyword => "null".into(),
        SyntaxKind::ObjectKeyword => "object".into(),
        SyntaxKind::NeverKeyword => "never".into(),
        SyntaxKind::TrueKeyword => "true".into(),
        SyntaxKind::FalseKeyword => "false".into(),
        SyntaxKind::UniqueKeyword => "unique".into(),
        SyntaxKind::ReadonlyKeyword => "readonly".into(),
        SyntaxKind::KeyOfKeyword => "keyof".into(),
        SyntaxKind::Identifier => node.text().to_string(),
        SyntaxKind::StringLiteral => {
            if let NodeData::StringLiteral(d) = &node.data {
                format!("\"{}\"", d.text)
            } else {
                "?".into()
            }
        }
        SyntaxKind::NumericLiteral => {
            if let NodeData::NumericLiteral(d) = &node.data {
                d.text.clone()
            } else {
                "?".into()
            }
        }
        SyntaxKind::BigIntLiteral => {
            if let NodeData::BigIntLiteral(d) = &node.data {
                d.text.clone()
            } else {
                "?".into()
            }
        }
        SyntaxKind::LiteralType => {
            if let NodeData::LiteralTypeNode(d) = &node.data {
                type_node_to_string(&d.literal)
            } else {
                "?".into()
            }
        }
        SyntaxKind::TypeReference => {
            if let NodeData::TypeReferenceNode(d) = &node.data {
                let name = type_node_to_string(&d.type_name);
                if let Some(args) = &d.type_arguments {
                    let parts: Vec<String> = args.nodes.iter().map(type_node_to_string).collect();
                    format!("{}<{}>", name, parts.join(", "))
                } else {
                    name
                }
            } else {
                "?".into()
            }
        }
        SyntaxKind::ArrayType => {
            if let NodeData::ArrayTypeNode(d) = &node.data {
                format!("{}[]", type_node_to_string(&d.element_type))
            } else {
                "?".into()
            }
        }
        SyntaxKind::TupleType => {
            if let NodeData::TupleTypeNode(d) = &node.data {
                let parts: Vec<String> = d.elements.nodes.iter().map(type_node_to_string).collect();
                format!("[{}]", parts.join(", "))
            } else {
                "?".into()
            }
        }
        SyntaxKind::UnionType => {
            if let NodeData::UnionTypeNode(d) = &node.data {
                let parts: Vec<String> = d.types.nodes.iter().map(type_node_to_string).collect();
                parts.join(" | ")
            } else {
                "?".into()
            }
        }
        SyntaxKind::IntersectionType => {
            if let NodeData::IntersectionTypeNode(d) = &node.data {
                let parts: Vec<String> = d.types.nodes.iter().map(type_node_to_string).collect();
                parts.join(" & ")
            } else {
                "?".into()
            }
        }
        SyntaxKind::ParenthesizedType => {
            if let NodeData::ParenthesizedTypeNode(d) = &node.data {
                format!("({})", type_node_to_string(&d.type_node))
            } else {
                "?".into()
            }
        }
        SyntaxKind::FunctionType => {
            if let NodeData::FunctionTypeNode(d) = &node.data {
                let params: Vec<String> =
                    d.parameters.nodes.iter().map(type_node_to_string).collect();
                let ret = d
                    .type_node
                    .as_ref()
                    .map(type_node_to_string)
                    .unwrap_or_else(|| "unknown".into());
                format!("({}) => {}", params.join(", "), ret)
            } else {
                "?".into()
            }
        }
        SyntaxKind::Parameter => {
            if let NodeData::ParameterDeclaration(d) = &node.data {
                let name = type_node_to_string(&d.name);
                let ty = d
                    .type_node
                    .as_ref()
                    .map(type_node_to_string)
                    .unwrap_or_else(|| "any".into());
                if d.question_token.is_some() {
                    format!("{}?: {}", name, ty)
                } else {
                    format!("{}: {}", name, ty)
                }
            } else {
                "?".into()
            }
        }
        SyntaxKind::TypeLiteral => {
            if let NodeData::TypeLiteralNode(d) = &node.data {
                let members: Vec<String> =
                    d.members.nodes.iter().map(type_node_to_string).collect();
                if members.is_empty() {
                    "{}".into()
                } else {
                    format!("{{ {}; }}", members.join("; "))
                }
            } else {
                "?".into()
            }
        }
        SyntaxKind::PropertySignature => {
            if let NodeData::PropertySignatureDeclaration(d) = &node.data {
                let name = type_node_to_string(&d.name);
                let ty = type_node_to_string(&d.type_node);
                if d.postfix_token.is_some() {
                    format!("{}?: {}", name, ty)
                } else {
                    format!("{}: {}", name, ty)
                }
            } else {
                "?".into()
            }
        }
        SyntaxKind::RestType => {
            if let NodeData::RestTypeNode(d) = &node.data {
                format!("...{}", type_node_to_string(&d.type_node))
            } else {
                "?".into()
            }
        }
        SyntaxKind::TypeOperator => {
            if let NodeData::TypeOperatorNode(d) = &node.data {
                let op = match d.operator {
                    SyntaxKind::UniqueKeyword => "unique ",
                    SyntaxKind::ReadonlyKeyword => "readonly ",
                    SyntaxKind::KeyOfKeyword => "keyof ",
                    _ => "",
                };
                format!("{}{}", op, type_node_to_string(&d.type_node))
            } else {
                "?".into()
            }
        }
        _ => "?".into(),
    }
}

fn assert_var_type_round_trips(source: &str) {
    let mut checker = build_checker(source);
    let type_node = first_var_type_node(&checker);
    let t = checker.get_type_from_type_node(&type_node);
    let expected = checker.type_to_string(&t);
    let built = checker.type_to_type_node(&t);
    let actual = type_node_to_string(&built);
    assert_eq!(
        actual, expected,
        "type_to_type_node round-trip mismatch for source: {source}\n\
             type_to_string: {expected:?}\n\
             type_node_to_string: {actual:?}"
    );
}

#[test]
fn type_to_type_node_number() {
    assert_var_type_round_trips("let x: number = 0;");
}

#[test]
fn type_to_type_node_string() {
    assert_var_type_round_trips("let x: string = \"\";");
}

#[test]
fn type_to_type_node_boolean() {
    assert_var_type_round_trips("let x: boolean = true;");
}

#[test]
fn type_to_type_node_void() {
    assert_var_type_round_trips("let x: void = undefined;");
}

#[test]
fn type_to_type_node_any() {
    assert_var_type_round_trips("let x: any = 0;");
}

#[test]
fn type_to_type_node_unknown() {
    assert_var_type_round_trips("let x: unknown = 0;");
}

#[test]
fn type_to_type_node_never() {
    assert_var_type_round_trips("let x: never;");
}

#[test]
fn type_to_type_node_null() {
    assert_var_type_round_trips("let x: null = null;");
}

#[test]
fn type_to_type_node_undefined() {
    assert_var_type_round_trips("let x: undefined = undefined;");
}

#[test]
fn type_to_type_node_array_of_number() {
    assert_var_type_round_trips("let x: number[] = [];");
}

#[test]
fn type_to_type_node_array_of_string() {
    assert_var_type_round_trips("let x: string[] = [\"\"];");
}

#[test]
fn type_to_type_node_tuple() {
    assert_var_type_round_trips("let x: [number, string] = [0, \"\"];");
}

#[test]
fn type_to_type_node_union_number_string() {
    assert_var_type_round_trips("let x: number | string = 0;");
}

#[test]
fn type_to_type_node_union_string_null() {
    assert_var_type_round_trips("let x: string | null = null;");
}

#[test]
fn type_to_type_node_intersection() {
    assert_var_type_round_trips(
        "interface A { a: number }\n\
             interface B { b: string }\n\
             let x: A & B = { a: 1, b: \"\" };",
    );
}

#[test]
fn type_to_type_node_generic_interface_reference() {
    assert_var_type_round_trips(
        "interface Foo<T> { value: T }\n\
             let x: Foo<number> = { value: 1 };",
    );
}

#[test]
fn type_to_type_node_function_type() {
    assert_var_type_round_trips("let x: (a: number) => string = (a) => \"\";");
}

#[test]
fn type_to_type_node_object_literal() {
    assert_var_type_round_trips("let x: { a: number; b: string } = { a: 1, b: \"\" };");
}

#[test]
fn type_to_type_node_string_literal_type() {
    assert_var_type_round_trips("let x: \"hello\" = \"hello\";");
}

#[test]
fn type_to_type_node_numeric_literal_type() {
    assert_var_type_round_trips("let x: 42 = 42;");
}

use crate::ast::node_data_generated::for_each_child;

fn find_identifier(node: &Arc<Node>, name: &str) -> Option<Arc<Node>> {
    if node.kind == SyntaxKind::Identifier {
        if let NodeData::Identifier(id) = &node.data {
            if id.text == name {
                return Some(Arc::clone(node));
            }
        }
    }
    let mut found: Option<Arc<Node>> = None;
    for_each_child(node, |child| {
        if found.is_none() {
            found = find_identifier(child, name);
        }
        found.is_some()
    });
    found
}

fn display_parts_for(source: &str, name: &str) -> Vec<SymbolDisplayPart> {
    let mut checker = build_checker(source);
    let file = checker
        .files
        .iter()
        .find(|f| f.file_name == "/proj/entry.ts")
        .expect("entry source file");
    let node = find_identifier(&file.node, name)
        .unwrap_or_else(|| panic!("identifier `{name}` not found in source:\n{source}"));
    checker.get_quick_info_display_parts(&node)
}

fn parts_text(parts: &[SymbolDisplayPart]) -> String {
    parts.iter().map(|p| p.text.as_str()).collect()
}

#[test]
fn display_parts_function() {
    let parts = display_parts_for("function foo(x: number): string { return \"\"; }", "foo");

    assert_eq!(parts_text(&parts), "function foo(x: number): string");

    assert_eq!(
        parts,
        vec![
            SymbolDisplayPart::new("function", DisplayPartKind::Keyword),
            SymbolDisplayPart::new(" ", DisplayPartKind::Space),
            SymbolDisplayPart::new("foo", DisplayPartKind::FunctionName),
            SymbolDisplayPart::new("(", DisplayPartKind::Punctuation),
            SymbolDisplayPart::new("x", DisplayPartKind::ParameterName),
            SymbolDisplayPart::new(": ", DisplayPartKind::Space),
            SymbolDisplayPart::new("number", DisplayPartKind::Keyword),
            SymbolDisplayPart::new(")", DisplayPartKind::Punctuation),
            SymbolDisplayPart::new(": ", DisplayPartKind::Space),
            SymbolDisplayPart::new("string", DisplayPartKind::Keyword),
        ]
    );
}

#[test]
fn display_parts_function_two_params() {
    let parts = display_parts_for(
        "function f(a: string, b: number): boolean { return true; }",
        "f",
    );
    assert_eq!(
        parts_text(&parts),
        "function f(a: string, b: number): boolean"
    );
}

#[test]
fn display_parts_let_variable() {
    let parts = display_parts_for("let x: number = 0;", "x");
    assert_eq!(parts_text(&parts), "let x: number");
    assert_eq!(
        parts[0],
        SymbolDisplayPart::new("let", DisplayPartKind::Keyword)
    );
    assert_eq!(
        parts[2],
        SymbolDisplayPart::new("x", DisplayPartKind::VariableName)
    );
    assert_eq!(
        parts[4],
        SymbolDisplayPart::new("number", DisplayPartKind::Keyword)
    );
}

#[test]
fn display_parts_const_variable() {
    let parts = display_parts_for("const s: string = \"hi\";", "s");
    assert_eq!(parts_text(&parts), "const s: string");
    assert_eq!(
        parts[0],
        SymbolDisplayPart::new("const", DisplayPartKind::Keyword)
    );
}

#[test]
fn display_parts_var_variable() {
    let parts = display_parts_for("var v: boolean = true;", "v");
    assert_eq!(parts_text(&parts), "var v: boolean");
}

#[test]
fn display_parts_class() {
    let parts = display_parts_for("class Foo<T, U> {}", "Foo");
    assert_eq!(parts_text(&parts), "class Foo<T, U>");
    assert_eq!(
        parts[0],
        SymbolDisplayPart::new("class", DisplayPartKind::Keyword)
    );
    assert_eq!(
        parts[2],
        SymbolDisplayPart::new("Foo", DisplayPartKind::ClassName)
    );

    assert_eq!(
        parts[4],
        SymbolDisplayPart::new("T", DisplayPartKind::TypeParameterName)
    );
    assert_eq!(
        parts[6],
        SymbolDisplayPart::new("U", DisplayPartKind::TypeParameterName)
    );
}

#[test]
fn display_parts_interface() {
    let parts = display_parts_for("interface Bar<T> { x: T; }", "Bar");
    assert_eq!(parts_text(&parts), "interface Bar<T>");
    assert_eq!(
        parts[0],
        SymbolDisplayPart::new("interface", DisplayPartKind::Keyword)
    );
    assert_eq!(
        parts[2],
        SymbolDisplayPart::new("Bar", DisplayPartKind::InterfaceName)
    );
}

#[test]
fn display_parts_enum() {
    let parts = display_parts_for("enum Color { Red, Green, Blue }", "Color");
    assert_eq!(parts_text(&parts), "enum Color");
    assert_eq!(
        parts[0],
        SymbolDisplayPart::new("enum", DisplayPartKind::Keyword)
    );
    assert_eq!(
        parts[2],
        SymbolDisplayPart::new("Color", DisplayPartKind::EnumName)
    );
}

#[test]
fn display_parts_type_alias() {
    let parts = display_parts_for("type MyNumber = number;", "MyNumber");
    assert_eq!(parts_text(&parts), "type MyNumber = number");
    assert_eq!(
        parts[0],
        SymbolDisplayPart::new("type", DisplayPartKind::Keyword)
    );

    assert_eq!(parts.last().unwrap().kind, DisplayPartKind::Keyword);
}

#[test]
fn display_parts_type_alias_with_type_params() {
    let parts = display_parts_for("type Id<T> = T;", "Id");
    assert!(parts_text(&parts).starts_with("type Id<T> = "));
}

#[test]
fn display_parts_kind_round_trips_to_strings() {
    assert_eq!(DisplayPartKind::Keyword.as_str(), "keyword");
    assert_eq!(DisplayPartKind::FunctionName.as_str(), "functionName");
    assert_eq!(DisplayPartKind::ClassName.as_str(), "className");
    assert_eq!(DisplayPartKind::ParameterName.as_str(), "parameterName");
    assert_eq!(DisplayPartKind::Punctuation.as_str(), "punctuation");
    assert_eq!(DisplayPartKind::Space.as_str(), "space");
}

#[test]
fn type_to_display_parts_intrinsic_keyword() {
    let mut checker = build_checker("let x: number = 0;");
    let type_node = first_var_type_node(&checker);
    let t = checker.get_type_from_type_node(&type_node);
    let parts = checker.type_to_display_parts(&t);
    assert_eq!(
        parts,
        vec![SymbolDisplayPart::new("number", DisplayPartKind::Keyword)]
    );
}

#[test]
fn type_to_display_parts_class_name() {
    let mut checker = build_checker("class Foo {}\nlet x: Foo = new Foo();");
    let type_node = first_var_type_node(&checker);
    let t = checker.get_type_from_type_node(&type_node);
    let parts = checker.type_to_display_parts(&t);
    assert!(!parts.is_empty());
}
