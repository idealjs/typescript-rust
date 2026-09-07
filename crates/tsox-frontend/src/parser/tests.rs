use super::*;

#[test]
fn parse_identifier() {
    let mut p = Parser::new("foo");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::Identifier);
    assert_eq!(node.text(), "foo");
    assert!(p.diagnostics().is_empty());
}

#[test]
fn parse_numeric_literal() {
    let mut p = Parser::new("42");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::NumericLiteral);
    assert_eq!(node.text(), "42");
}

#[test]
fn parse_string_literal() {
    let mut p = Parser::new("\"hello\"");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::StringLiteral);
}

#[test]
fn parse_private_identifier_class_field() {
    let (_, diags) = Parser::parse_source_file_text_with_diagnostics(
        "a.ts",
        "class C { #name: string; }".to_string(),
    );
    assert!(
        diags.is_empty(),
        "expected no diagnostics, got: {:?}",
        diags
    );
}

#[test]
fn parse_private_identifier_member_access() {
    let mut p = Parser::new("this.#name");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::PropertyAccessExpression);
    assert!(
        p.diagnostics().is_empty(),
        "expected no diagnostics, got: {:?}",
        p.diagnostics()
    );
}

#[test]
fn parse_less_than_is_comparison_not_type_args() {
    let mut p = Parser::new("if (x < 10) { }");
    let _ = p.parse_expression();
    assert!(
        p.diagnostics().iter().all(|d| {
            let msg = format!("{}", d.message);
            !msg.contains("expected")
        }),
        "expected no 'expected' diagnostics, got: {:?}",
        p.diagnostics()
    );
}

#[test]
fn parse_generic_call_keeps_type_arguments() {
    let mut p = Parser::new("f<string>(x)");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::CallExpression);
    assert!(
        p.diagnostics().is_empty(),
        "expected no diagnostics, got: {:?}",
        p.diagnostics()
    );
}

#[test]
fn parse_generic_arrow_function() {
    let mut p = Parser::new("<T>(x: T): T => x");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::ArrowFunction);
    assert!(
        p.diagnostics().is_empty(),
        "expected no diagnostics, got: {:?}",
        p.diagnostics()
    );
}

#[test]
fn parse_async_generic_arrow_function() {
    let mut p = Parser::new("async <T>(value: T): T => value");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::ArrowFunction);
    assert!(
        p.diagnostics().is_empty(),
        "expected no diagnostics, got: {:?}",
        p.diagnostics()
    );
}

#[test]
fn parse_generic_arrow_not_confused_with_comparison() {
    let mut p = Parser::new("let r = a < b;");
    let _ = p.parse_expression();
    assert!(
        p.diagnostics().iter().all(|d| {
            let msg = format!("{}", d.message);
            !msg.contains("expected")
        }),
        "expected no 'expected' diagnostics, got: {:?}",
        p.diagnostics()
    );
}

#[test]
fn parse_for_loop_condition_less_than() {
    let (_, diags) = Parser::parse_source_file_text_with_diagnostics(
        "a.ts",
        "function f() { for (let i = 0; i < n; i++) { } }".to_string(),
    );
    assert!(
        diags.iter().all(|d| {
            let msg = format!("{}", d.message);
            !msg.contains("expected")
        }),
        "expected no 'expected' diagnostics, got: {:?}",
        diags
    );
}

#[test]
fn parse_multi_declarator_variable_list() {
    let (_, diags) = Parser::parse_source_file_text_with_diagnostics(
        "a.ts",
        "let a = 1, b = 2, c = 3;\na; b; c;".to_string(),
    );
    assert!(
        diags.iter().all(|d| {
            let msg = format!("{}", d.message);
            !msg.contains("expected")
        }),
        "expected no parse errors, got: {:?}",
        diags
    );
}

#[test]
fn parse_parenthesized() {
    let mut p = Parser::new("(foo)");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::ParenthesizedExpression);
    assert!(p.diagnostics().is_empty());
}

#[test]
fn parse_unary() {
    let mut p = Parser::new("!foo");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::PrefixUnaryExpression);
}

#[test]
fn parse_binary_precedence() {
    let mut p = Parser::new("a + b * c");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::BinaryExpression);
}

#[test]
fn parse_var_statement() {
    let mut p = Parser::new("var x = 1;");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::VariableStatement);
}

#[test]
fn parse_let_statement() {
    let mut p = Parser::new("let x: number = 42;");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::VariableStatement);
}

#[test]
fn parse_declare_variable_statement() {
    let mut p = Parser::new("declare var x: string;");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::VariableStatement);
    assert!(p.diagnostics().is_empty());
}

#[test]
fn parse_declare_function_statement() {
    let mut p = Parser::new("declare function f(): void;");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::FunctionDeclaration);
    assert!(p.diagnostics().is_empty());
}

#[test]
fn parse_declare_type_alias_statement() {
    let mut p = Parser::new("declare type Name = string;");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::TypeAliasDeclaration);
    assert!(p.diagnostics().is_empty());
}

#[test]
fn parse_export_declare_interface_statement() {
    let mut p = Parser::new("export declare interface Box { value: string; }");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::InterfaceDeclaration);
    assert!(p.diagnostics().is_empty());
}

#[test]
fn parse_if_statement() {
    let mut p = Parser::new("if (x) { y; }");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::IfStatement);
}

#[test]
fn parse_if_else_statement() {
    let mut p = Parser::new("if (x) { y; } else { z; }");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::IfStatement);
}

#[test]
fn parse_return_statement() {
    let mut p = Parser::new("return 42;");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::ReturnStatement);
}

#[test]
fn parse_return_void() {
    let mut p = Parser::new("return;");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::ReturnStatement);
}

#[test]
fn parse_while_statement() {
    let mut p = Parser::new("while (true) { x; }");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::WhileStatement);
}

#[test]
fn parse_for_statement() {
    let mut p = Parser::new("for (let i = 0; i < 10; i++) { x; }");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::ForStatement);
}

#[test]
fn parse_break_statement() {
    let mut p = Parser::new("break;");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::BreakStatement);
}

#[test]
fn parse_continue_statement() {
    let mut p = Parser::new("continue;");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::ContinueStatement);
}

#[test]
fn parse_throw_statement() {
    let mut p = Parser::new("throw new Error();");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::ThrowStatement);
}

#[test]
fn parse_block() {
    let mut p = Parser::new("{ x; y; }");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::Block);
}

#[test]
fn parse_empty_statement() {
    let mut p = Parser::new(";");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::EmptyStatement);
}

#[test]
fn parse_debugger_statement() {
    let mut p = Parser::new("debugger;");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::DebuggerStatement);
}

#[test]
fn parse_member_access() {
    let mut p = Parser::new("a.b.c");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::PropertyAccessExpression);
}

#[test]
fn parse_call_expression() {
    let mut p = Parser::new("foo(1, 2)");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::CallExpression);
}

#[test]
fn parse_array_literal() {
    let mut p = Parser::new("[1, 2, 3]");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::ArrayLiteralExpression);
}

#[test]
fn parse_object_literal() {
    let mut p = Parser::new("{ a: 1, b: 2 }");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::ObjectLiteralExpression);
}

#[test]
fn parse_assignment_expression() {
    let mut p = Parser::new("x = 42");
    let node = p.parse_expression();
    assert_eq!(node.kind, SyntaxKind::BinaryExpression);
}

#[test]
fn script_kind_from_file_name_matches_go_mapping() {
    assert_eq!(script_kind_from_file_name("a.ts"), ScriptKind::Ts);
    assert_eq!(script_kind_from_file_name("a.mts"), ScriptKind::Ts);
    assert_eq!(script_kind_from_file_name("a.cts"), ScriptKind::Ts);
    assert_eq!(script_kind_from_file_name("a.tsx"), ScriptKind::Tsx);
    assert_eq!(script_kind_from_file_name("a.js"), ScriptKind::Js);
    assert_eq!(script_kind_from_file_name("a.mjs"), ScriptKind::Js);
    assert_eq!(script_kind_from_file_name("a.cjs"), ScriptKind::Js);
    assert_eq!(script_kind_from_file_name("a.jsx"), ScriptKind::Jsx);
    assert_eq!(script_kind_from_file_name("a.json"), ScriptKind::Json);
    assert_eq!(script_kind_from_file_name("a.txt"), ScriptKind::Unknown);

    let tsx = Parser::parse_source_file_text("a.tsx", "const x = <div />;".to_string());
    assert_eq!(tsx.script_kind, ScriptKind::Tsx);
    assert_eq!(tsx.language_variant, LanguageVariant::Jsx);

    let jsx = Parser::parse_source_file_text("a.jsx", "const x = <div />;".to_string());
    assert_eq!(jsx.script_kind, ScriptKind::Jsx);
    assert_eq!(jsx.language_variant, LanguageVariant::Jsx);
}

#[test]
fn namespace_import_is_wrapped_in_import_clause() {
    let mut p = Parser::new("import * as ns from \"mod\";");
    let node = p.parse_statement();
    let import = match &node.data {
        NodeData::ImportDeclaration(data) => data,
        other => panic!("expected import declaration, got {other:?}"),
    };
    let clause = import
        .import_clause
        .as_ref()
        .expect("missing import clause");
    let clause_data = match &clause.data {
        NodeData::ImportClause(data) => data,
        other => panic!("expected import clause, got {other:?}"),
    };
    assert!(clause_data.name.is_none());
    let named_bindings = clause_data
        .named_bindings
        .as_ref()
        .expect("missing namespace import");
    assert_eq!(named_bindings.kind, SyntaxKind::NamespaceImport);
}

fn first_import_specifier(source: &str) -> (bool, Option<String>, String) {
    let (_file, diags) =
        Parser::parse_source_file_text_with_diagnostics("a.ts", source.to_string());
    assert!(diags.is_empty(), "{source}: {diags:?}");
    let file = &_file;
    let stmt = match &file.node.data {
        NodeData::SourceFile(d) => d.statements.nodes[0].clone(),
        other => panic!("expected source file, got {other:?}"),
    };
    let import = match &stmt.data {
        NodeData::ImportDeclaration(d) => d,
        other => panic!("expected import declaration, got {other:?}"),
    };
    let clause = match &import.import_clause.as_ref().unwrap().data {
        NodeData::ImportClause(d) => d,
        other => panic!("expected import clause, got {other:?}"),
    };
    let named = match &clause.named_bindings.as_ref().unwrap().data {
        NodeData::NamedImports(d) => d,
        other => panic!("expected named imports, got {other:?}"),
    };
    match &named.elements.nodes[0].data {
        NodeData::ImportSpecifier(d) => (
            d.is_type_only,
            d.property_name.as_ref().map(|p| p.text().to_string()),
            d.name.text().to_string(),
        ),
        other => panic!("expected import specifier, got {other:?}"),
    }
}

#[test]
fn specifier_bare_type_is_the_name() {
    assert_eq!(
        first_import_specifier("import { type } from \"mod\";"),
        (false, None, "type".to_string())
    );
    let (_file, diags) = Parser::parse_source_file_text_with_diagnostics(
        "a.ts",
        "export { type };\nexport {};\n".to_string(),
    );
    assert!(diags.is_empty(), "{diags:?}");
}

#[test]
fn specifier_type_as_shapes() {
    assert_eq!(
        first_import_specifier("import { type as } from \"mod\";"),
        (true, None, "as".to_string())
    );

    assert_eq!(
        first_import_specifier("import { type as as } from \"mod\";"),
        (false, Some("type".to_string()), "as".to_string())
    );

    assert_eq!(
        first_import_specifier("import { type as as as } from \"mod\";"),
        (true, Some("as".to_string()), "as".to_string())
    );

    assert_eq!(
        first_import_specifier("import { type x } from \"mod\";"),
        (true, None, "x".to_string())
    );

    assert_eq!(
        first_import_specifier("import { type x as y } from \"mod\";"),
        (true, Some("x".to_string()), "y".to_string())
    );
}

#[test]
fn import_type_named_imports_use_phase_modifier() {
    let mut p = Parser::new("import type { A, B as C } from \"mod\";");
    let node = p.parse_statement();
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let import = match &node.data {
        NodeData::ImportDeclaration(data) => data,
        other => panic!("expected import declaration, got {other:?}"),
    };
    let clause = import
        .import_clause
        .as_ref()
        .expect("missing import clause");
    let clause_data = match &clause.data {
        NodeData::ImportClause(data) => data,
        other => panic!("expected import clause, got {other:?}"),
    };
    assert_eq!(clause_data.phase_modifier, Some(SyntaxKind::TypeKeyword));
    assert!(clause_data.name.is_none());
    let named_bindings = clause_data
        .named_bindings
        .as_ref()
        .expect("missing named imports");
    assert_eq!(named_bindings.kind, SyntaxKind::NamedImports);
}

#[test]
fn import_type_multiline_named_imports() {
    let source = "import type {\n  A,\n  B,\n} from \"mod\";";
    let (_file, diagnostics) =
        Parser::parse_source_file_text_with_diagnostics("a.ts", source.to_string());
    assert!(diagnostics.is_empty(), "{:?}", diagnostics);
}

#[test]
fn import_default_named_type_is_not_phase_modifier() {
    let mut p = Parser::new("import type from \"mod\";");
    let node = p.parse_statement();
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let import = match &node.data {
        NodeData::ImportDeclaration(data) => data,
        other => panic!("expected import declaration, got {other:?}"),
    };
    let clause = import
        .import_clause
        .as_ref()
        .expect("missing import clause");
    let clause_data = match &clause.data {
        NodeData::ImportClause(data) => data,
        other => panic!("expected import clause, got {other:?}"),
    };
    assert_eq!(clause_data.phase_modifier, None);
    assert!(clause_data.name.is_some());
}

#[test]
fn import_equals_declaration_matches_go_entry_split() {
    let mut p = Parser::new("import type A = B.C;");
    let node = p.parse_statement();
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let import_equals = match &node.data {
        NodeData::ImportEqualsDeclaration(data) => data,
        other => panic!("expected import equals declaration, got {other:?}"),
    };
    assert!(import_equals.is_type_only);
    assert_eq!(import_equals.name.text(), "A");
    assert_eq!(
        import_equals.module_reference.kind,
        SyntaxKind::QualifiedName
    );
}

#[test]
fn record_warn4_import_blocks_from_ai_color_toner_parse() {
    let cases = [
        (
            "AiModelField.tsx",
            "import type { AiProfile } from './types'\n",
        ),
        (
            "AiTestControls.tsx",
            "import type {\n  AiProfile,\n  AiProfileOperation,\n  AiTestMode,\n  AiTestResult,\n} from './types'\n",
        ),
        (
            "App.tsx",
            "import { ColorFloatPanel, ModalLayer, TokenFloatPanel } from './AppPanels'\n\
                 import { ComposerPage } from './ComposerPage'\n\
                 import { PalettePage } from './PalettePage'\n\
                 import { SettingsPage } from './SettingsPage'\n\
                 import { ThemesPage } from './ThemesPage'\n\
                 import { useAppController } from './useAppController'\n\
                 import './App.css'\n\
                 import type { Page } from './appTypes'\n",
        ),
        (
            "previewGeneration.ts",
            "import { findToken } from './colorRefs'\n\
                 import { generateAiText } from './aiAdapters'\n\
                 import { canActivateProfile } from './aiProfiles'\n\
                 import { extractAndValidatePreviewHtml } from './previewValidation'\n\
                 import type { AiProfile, AppState, Theme } from './types'\n",
        ),
    ];

    for (file_name, source) in cases {
        let (_file, diagnostics) =
            Parser::parse_source_file_text_with_diagnostics(file_name, source.to_string());
        assert!(
            diagnostics.is_empty(),
            "{file_name} produced diagnostics: {diagnostics:?}"
        );
    }
}

#[test]
fn record_warn6_arrow_and_as_const_fragments_parse() {
    let cases = [
        (
            "AiModelField.test.tsx",
            "describe('AiModelField', () => {\n  it('renders', () => {\n    const profile = {\n      model: 'manual-model',\n      models: [\n        { id: 'model-a', name: 'Model A' },\n        { id: 'model-b', name: 'Model B' },\n      ],\n    }\n  })\n})\n",
        ),
        (
            "AiTestControls.test.tsx",
            "describe('AiTestControls', () => {\n  it('renders', () => {\n    const profile = {\n      lastQuickTest: {\n        status: 'success' as const,\n        latencyMs: 91,\n      },\n    }\n  })\n})\n",
        ),
        (
            "previewGeneration.ts",
            "export function builtinTemplate(state: AppState) {\n  const legend = state.tokenNames\n    .map(\n      (name) => `<div style=\"color:${muted}\"><span style=\"background:var(--${name})\"></span>--${name}</div>`,\n    )\n    .join('')\n\n  return `<body style=\"background:${bg};color:${text}\">\n    ${['Design', 'Build', 'Verify']\n      .map(\n        (title) => `<div>${title}</div>`,\n      )\n      .join('')}\n  </body>`\n}\n",
        ),
        (
            "previewGeneration.ts",
            "export async function aiGenerate(\n  state: AppState,\n  intent: string,\n  deps: PreviewGenerationDeps = {},\n) {\n  const data = await (deps.generateText || generateAiText)(\n    profile,\n    systemPrompt,\n    intent,\n  )\n  return data\n}\n",
        ),
    ];

    for (file_name, source) in cases {
        let (_file, diagnostics) =
            Parser::parse_source_file_text_with_diagnostics(file_name, source.to_string());
        assert!(
            diagnostics.is_empty(),
            "{file_name} produced diagnostics: {diagnostics:?}"
        );
    }
}

#[test]
fn record_warn6_tsx_jsx_fragment_from_app_parse() {
    let source = "function App() {\n  const controller = useAppController()\n\n  return (\n    <div className=\"app-shell\" onMouseDown={() => controller.setFloatPanel(null)}>\n      <nav className=\"top-nav\" onMouseDown={(event) => event.stopPropagation()}>\n        <button\n          className=\"brand\"\n          type=\"button\"\n          onClick={() => controller.navigate('palette')}\n        >\n          <span>COLOR</span>\n          <span>TONER</span>\n        </button>\n        {NAV_ITEMS.map(([itemPage, label]) => (\n          <button\n            className={controller.page === itemPage ? 'active' : ''}\n            key={itemPage}\n            type=\"button\"\n            onClick={() => controller.navigate(itemPage)}\n          >\n            {label}\n          </button>\n        ))}\n      </nav>\n    </div>\n  )\n}\n";
    let (_file, diagnostics) =
        Parser::parse_source_file_text_with_diagnostics("App.tsx", source.to_string());
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn parse_jsx_simple_element() {
    let source = "const x = <div>hello</div>;";
    let (_file, diagnostics) =
        Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn parse_jsx_fragment() {
    let source = "const x = <>fragment text</>;";
    let (_file, diagnostics) =
        Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn parse_jsx_self_closing() {
    let source = "const x = <img src=\"foo.png\" alt=\"bar\" />;";
    let (_file, diagnostics) =
        Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn parse_jsx_dashed_tag_name() {
    let source = "const x = <my-component data-foo=\"bar\">text</my-component>;";
    let (_file, diagnostics) =
        Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn parse_jsx_expression_children() {
    let source = "const x = <div>{items.map(i => <span key={i}>{i}</span>)}</div>;";
    let (_file, diagnostics) =
        Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn parse_jsx_nested_elements() {
    let source = "const x = <div><p><span>deep</span></p></div>;";
    let (_file, diagnostics) =
        Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn parse_jsx_spread_attribute() {
    let source = "const x = <div {...props}>text</div>;";
    let (_file, diagnostics) =
        Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn parse_jsx_member_expression_tag() {
    let source = "const x = <Foo.Bar>text</Foo.Bar>;";
    let (_file, diagnostics) =
        Parser::parse_source_file_text_with_diagnostics("test.tsx", source.to_string());
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn parse_primitive_keyword_type_nodes() {
    for (src, expected_kind) in [
        ("type T = any;", SyntaxKind::AnyKeyword),
        ("type T = unknown;", SyntaxKind::UnknownKeyword),
        ("type T = string;", SyntaxKind::StringKeyword),
        ("type T = number;", SyntaxKind::NumberKeyword),
        ("type T = bigint;", SyntaxKind::BigIntKeyword),
        ("type T = symbol;", SyntaxKind::SymbolKeyword),
        ("type T = boolean;", SyntaxKind::BooleanKeyword),
        ("type T = undefined;", SyntaxKind::UndefinedKeyword),
        ("type T = never;", SyntaxKind::NeverKeyword),
        ("type T = object;", SyntaxKind::ObjectKeyword),
        ("type T = void;", SyntaxKind::VoidKeyword),
    ] {
        let mut p = Parser::new(src);
        let node = p.parse_statement();
        assert_eq!(node.kind, SyntaxKind::TypeAliasDeclaration, "source: {src}");
        let alias = match &node.data {
            NodeData::TypeAliasDeclaration(data) => data,
            other => panic!("expected type alias, got {other:?} for {src}"),
        };
        assert_eq!(alias.type_node.kind, expected_kind, "source: {src}");
        assert!(
            matches!(alias.type_node.data, NodeData::KeywordTypeNode),
            "expected KeywordTypeNode for {src}"
        );
    }
}

#[test]
fn parse_keyword_type_followed_by_dot_is_type_reference() {
    let mut p = Parser::new("type T = String.fromCharCode;");
    let node = p.parse_statement();
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::TypeReference);
}

#[test]
fn parse_typeof_type_query() {
    let mut p = Parser::new("type T = typeof foo;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::TypeQuery);
}

#[test]
fn parse_import_type() {
    let mut p = Parser::new("type T = import(\"mod\").Foo;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::ImportType);
}

#[test]
fn parse_import_type_with_attributes() {
    let mut p =
        Parser::new("type T = import(\"pkg\", { with: { \"resolution-mode\": \"import\" } }).Foo;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::ImportType);
    let import_type = match &alias.type_node.data {
        NodeData::ImportTypeNode(data) => data,
        other => panic!("expected import type, got {other:?}"),
    };
    let attrs = import_type
        .attributes
        .as_ref()
        .expect("attributes clause present");
    match &attrs.data {
        NodeData::ImportAttributes(d) => {
            assert_eq!(d.token, SyntaxKind::WithKeyword);
            assert_eq!(d.attributes.len(), 1);
        }
        other => panic!("expected import attributes, got {other:?}"),
    }
}

#[test]
fn parse_import_type_missing_with_reports_1005() {
    let mut p = Parser::new("type T = import(\"pkg\", {\"resolution-mode\": \"require\"}).Foo;");
    let node = p.parse_statement();
    let diags = p.diagnostics();
    assert!(
        diags.iter().any(|d| d.message.code == 1005),
        "expected TS1005 'with' expected: {diags:?}"
    );
    assert_eq!(node.kind, SyntaxKind::TypeAliasDeclaration);
}

#[test]
fn parse_typeof_import_type() {
    let mut p = Parser::new("type T = typeof import(\"mod\").Foo;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::ImportType);
    let import_type = match &alias.type_node.data {
        NodeData::ImportTypeNode(data) => data,
        other => panic!("expected import type, got {other:?}"),
    };
    assert!(import_type.is_type_of);
}

#[test]
fn parse_negative_literal_type() {
    let mut p = Parser::new("type T = -1;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::LiteralType);
}

#[test]
fn parse_this_type() {
    let mut p = Parser::new("type T = this;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::ThisType);
}

#[test]
fn parse_tuple_types() {
    let mut p = Parser::new("type T = [string, number];");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::TupleType);

    let mut p = Parser::new("type T = readonly [string, number];");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::TypeOperator);

    let mut p = Parser::new("type T = [string, ...number[]];");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

    let mut p = Parser::new("type T = [name: string, age: number];");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::TupleType);

    let mut p = Parser::new("type T = [string?, number?];");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
}

#[test]
fn parse_union_intersection_precedence() {
    let mut p = Parser::new("type T = A | B & C;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };

    assert_eq!(alias.type_node.kind, SyntaxKind::UnionType);
    let union = match &alias.type_node.data {
        NodeData::UnionTypeNode(d) => d,
        other => panic!("expected union, got {other:?}"),
    };
    assert_eq!(union.types.nodes.len(), 2);

    assert_eq!(union.types.nodes[1].kind, SyntaxKind::IntersectionType);

    let mut p = Parser::new("type T = | A | B;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

    let mut p = Parser::new("type T = & A & B;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
}

#[test]
fn parse_generic_type_params_and_references() {
    let mut p = Parser::new("type T<A, B extends string = \"x\"> = A | B;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert!(alias.type_parameters.is_some());
    let tps = alias.type_parameters.as_ref().unwrap();
    assert_eq!(tps.nodes.len(), 2);
    assert_eq!(tps.nodes[0].kind, SyntaxKind::TypeParameter);
    assert_eq!(tps.nodes[1].kind, SyntaxKind::TypeParameter);

    let mut p = Parser::new("type T = Foo<string, number>;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::TypeReference);
    let tr = match &alias.type_node.data {
        NodeData::TypeReferenceNode(d) => d,
        other => panic!("expected type ref, got {other:?}"),
    };
    assert!(tr.type_arguments.is_some());
    assert_eq!(tr.type_arguments.as_ref().unwrap().nodes.len(), 2);

    let mut p = Parser::new("type T = A.B.C<T>;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

    let mut p = Parser::new("type T = Map<string, Array<number>>;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
}

#[test]
fn parse_mapped_types() {
    let mut p = Parser::new("type M<T> = { [K in keyof T]: string };");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::MappedType);

    let mut p = Parser::new("type M<T> = { readonly [K in keyof T]: string };");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

    let mut p = Parser::new("type M<T> = { -readonly [K in keyof T]: string };");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

    let mut p = Parser::new("type M<T> = { [K in keyof T]-?: string };");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

    let mut p = Parser::new("type M<T> = { [K in keyof T as `${K}`]: string };");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
}

#[test]
fn parse_conditional_types() {
    let mut p = Parser::new("type R<T> = T extends string ? number : boolean;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::ConditionalType);

    let mut p = Parser::new("type R<T> = T extends A ? X : T extends B ? Y : Z;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

    let mut p = Parser::new("type R<T> = T extends (infer U)[] ? U : never;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
}

#[test]
fn parse_call_and_construct_signatures() {
    let mut p = Parser::new("type T = { (): string };");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

    let mut p = Parser::new("type T = { new (): Foo };");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

    let mut p = Parser::new("type T = { abstract new (): Foo };");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

    let mut p = Parser::new("type T = () => string;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::FunctionType);

    let mut p = Parser::new("type T = new () => Foo;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
    let alias = match &node.data {
        NodeData::TypeAliasDeclaration(data) => data,
        other => panic!("expected type alias, got {other:?}"),
    };
    assert_eq!(alias.type_node.kind, SyntaxKind::ConstructorType);
}

#[test]
fn parse_index_signatures() {
    let mut p = Parser::new("type T = { [key: string]: number };");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

    let mut p = Parser::new("type T = { [index: number]: string };");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

    let mut p = Parser::new("type T = { readonly [key: string]: number };");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
}

#[test]
fn parse_satisfies_and_as_const() {
    let mut p = Parser::new("const x = { a: 1 } as const;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

    let mut p = Parser::new("const x = { a: 1 } satisfies Foo;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());

    let mut p = Parser::new("const x = foo!.bar;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics());
}

#[test]
fn parse_declare_module_string_literal() {
    let mut p = Parser::new("declare module \"foo\";");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ModuleDeclaration);
    let mod_decl = match &node.data {
        NodeData::ModuleDeclaration(d) => d,
        other => panic!("expected module decl, got {other:?}"),
    };
    assert_eq!(mod_decl.name.kind, SyntaxKind::StringLiteral);

    let mut p = Parser::new("declare module \"foo\" { export const x: number; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ModuleDeclaration);
    let mod_decl = match &node.data {
        NodeData::ModuleDeclaration(d) => d,
        other => panic!("expected module decl, got {other:?}"),
    };
    assert_eq!(mod_decl.name.kind, SyntaxKind::StringLiteral);
    assert!(mod_decl.body.is_some());
}

#[test]
fn parse_declare_namespace_dotted() {
    let mut p = Parser::new("declare namespace A.B.C { export const x: number; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ModuleDeclaration);
}

#[test]
fn parse_declare_global() {
    let mut p = Parser::new("declare global { const x: number; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ModuleDeclaration);
    let mod_decl = match &node.data {
        NodeData::ModuleDeclaration(d) => d,
        other => panic!("expected module decl, got {other:?}"),
    };
    assert_eq!(mod_decl.name.kind, SyntaxKind::Identifier);
    assert_eq!(mod_decl.name.text(), "global");
    assert!(mod_decl.body.is_some());
}

#[test]
fn parse_declare_class_full_body() {
    let mut p =
        Parser::new("declare class C extends Base { constructor(x: number); foo(): void; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ClassDeclaration);
}

#[test]
fn parse_declare_var_and_function() {
    let mut p = Parser::new("declare var x: number;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::VariableStatement);

    let mut p = Parser::new("declare const y: string;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::VariableStatement);

    let mut p = Parser::new("declare function f(): void;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::FunctionDeclaration);
    let fn_decl = match &node.data {
        NodeData::FunctionDeclaration(d) => d,
        other => panic!("expected function decl, got {other:?}"),
    };
    assert!(
        fn_decl.body.is_none(),
        "declare function should have no body"
    );
}

#[test]
fn parse_declare_enum_and_interface() {
    let mut p = Parser::new("declare enum E { A, B }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::EnumDeclaration);

    let mut p = Parser::new("declare interface I { foo(): void; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::InterfaceDeclaration);

    let mut p = Parser::new("declare type T = string;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::TypeAliasDeclaration);
}

#[test]
fn parse_asi_basic() {
    let mut p = Parser::new("let x = 1");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::VariableStatement);

    let mut p = Parser::new("let x = 1\nlet y = 2");
    let s1 = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(s1.kind, SyntaxKind::VariableStatement);
    let s2 = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(s2.kind, SyntaxKind::VariableStatement);

    let mut p = Parser::new("let x = 1;\nlet y = 2;");
    let s1 = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    let s2 = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(s1.kind, SyntaxKind::VariableStatement);
    assert_eq!(s2.kind, SyntaxKind::VariableStatement);
}

#[test]
fn parse_asi_postfix_no_line_break() {
    let mut p = Parser::new("let x = 1\n++y");
    let s1 = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(s1.kind, SyntaxKind::VariableStatement);

    let s2 = p.parse_statement();
    assert_eq!(s2.kind, SyntaxKind::ExpressionStatement);
}

#[test]
fn parse_asi_throw_needs_expression() {
    let mut p = Parser::new("throw\nnew Error()");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::ThrowStatement);
    let throw = match &node.data {
        NodeData::ThrowStatement(d) => d,
        other => panic!("expected throw, got {other:?}"),
    };

    assert_eq!(throw.expression.kind, SyntaxKind::Identifier);
    let id = match &throw.expression.data {
        NodeData::Identifier(d) => d,
        other => panic!("expected identifier, got {other:?}"),
    };
    assert!(
        id.text.is_empty(),
        "expected missing identifier, got {:?}",
        id.text
    );
}

#[test]
fn parse_scanner_errors_reach_parser_diagnostics() {
    let (file, diags) = Parser::parse_source_file_text_with_diagnostics("test.ts", "·".to_string());
    assert!(
        diags.iter().any(|d| d.message.code == 1127),
        "expected Invalid character diagnostic (TS1127), got: {diags:?}"
    );
    assert_eq!(file.node.kind, SyntaxKind::SourceFile);

    let (_file, diags) =
        Parser::parse_source_file_text_with_diagnostics("test.ts", "\"unterminated".to_string());
    assert!(
        diags.iter().any(|d| d.message.code == 1002),
        "expected Unterminated string literal diagnostic (TS1002), got: {diags:?}"
    );
}

#[test]
fn parse_regex_flag_diagnostics_reach_parser() {
    let (_file, diags) =
        Parser::parse_source_file_text_with_diagnostics("test.ts", "let x = /foo/z;".to_string());
    assert!(
        diags.iter().any(|d| d.message.code == 1499),
        "expected TS1499 for unknown regex flag, got: {diags:?}"
    );

    let (_file, diags) =
        Parser::parse_source_file_text_with_diagnostics("test.ts", "let x = /foo/gg;".to_string());
    assert!(
        diags.iter().any(|d| d.message.code == 1500),
        "expected TS1500 for duplicate regex flag, got: {diags:?}"
    );

    let (_file, diags) =
        Parser::parse_source_file_text_with_diagnostics("test.ts", "let x = /foo/uv;".to_string());
    assert!(
        diags.iter().any(|d| d.message.code == 1502),
        "expected TS1502 for u+v flags, got: {diags:?}"
    );

    let (_file, diags) =
        Parser::parse_source_file_text_with_diagnostics("test.ts", "let x = /foo/gim;".to_string());
    assert!(
        !diags
            .iter()
            .any(|d| matches!(d.message.code, 1499 | 1500 | 1501 | 1502)),
        "expected no regex flag diagnostics for valid flags, got: {diags:?}"
    );
}

#[test]
fn parse_import_attributes_with() {
    let mut p = Parser::new(r#"import x from "y" with { type: "json" }"#);
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ImportDeclaration);
    let imp = match &node.data {
        NodeData::ImportDeclaration(d) => d,
        other => panic!("expected import, got {other:?}"),
    };
    assert!(imp.attributes.is_some(), "expected import attributes");
    let attrs = imp.attributes.as_ref().unwrap();
    assert_eq!(attrs.kind, SyntaxKind::ImportAttributes);
    let attr_data = match &attrs.data {
        NodeData::ImportAttributes(d) => d,
        other => panic!("expected ImportAttributes, got {other:?}"),
    };
    assert_eq!(attr_data.token, SyntaxKind::WithKeyword);
    assert_eq!(attr_data.attributes.nodes.len(), 1);

    let mut p = Parser::new(r#"import { foo } from "y" with { type: "json", other: 42 }"#);
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ImportDeclaration);
    let imp = match &node.data {
        NodeData::ImportDeclaration(d) => d,
        other => panic!("expected import, got {other:?}"),
    };
    let attrs = imp.attributes.as_ref().unwrap();
    let attr_data = match &attrs.data {
        NodeData::ImportAttributes(d) => d,
        other => panic!("expected ImportAttributes, got {other:?}"),
    };
    assert_eq!(attr_data.attributes.nodes.len(), 2);

    let mut p = Parser::new(r#"export { foo } from "y" with { type: "json" }"#);
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ExportDeclaration);
    let exp = match &node.data {
        NodeData::ExportDeclaration(d) => d,
        other => panic!("expected export, got {other:?}"),
    };
    assert!(exp.attributes.is_some(), "expected export attributes");
}

#[test]
fn parse_decorators() {
    let mut p = Parser::new("@decorator\nclass Foo {}");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ClassDeclaration);
    let class = match &node.data {
        NodeData::ClassDeclaration(d) => d,
        other => panic!("expected class, got {other:?}"),
    };
    let mods = class
        .modifiers
        .as_ref()
        .expect("expected modifiers with decorator");
    assert!(mods.modifier_flags.contains(ModifierFlags::Decorator));
    let decorators: Vec<_> = mods
        .iter()
        .filter(|n| n.kind == SyntaxKind::Decorator)
        .collect();
    assert_eq!(decorators.len(), 1);

    let mut p = Parser::new("class Foo { @decorator bar() {} }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ClassDeclaration);
    let class = match &node.data {
        NodeData::ClassDeclaration(d) => d,
        other => panic!("expected class, got {other:?}"),
    };
    let members = &class.members;
    assert_eq!(members.nodes.len(), 1);
    let method = &members.nodes[0];
    assert_eq!(method.kind, SyntaxKind::MethodDeclaration);
    let method_data = match &method.data {
        NodeData::MethodDeclaration(d) => d,
        other => panic!("expected method, got {other:?}"),
    };
    let mods = method_data
        .modifiers
        .as_ref()
        .expect("method should have decorator modifiers");
    assert!(mods.modifier_flags.contains(ModifierFlags::Decorator));

    let mut p = Parser::new("class Foo { @decorator x: number = 1; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    let class = match &node.data {
        NodeData::ClassDeclaration(d) => d,
        other => panic!("expected class, got {other:?}"),
    };
    let prop = &class.members.nodes[0];
    assert_eq!(prop.kind, SyntaxKind::PropertyDeclaration);

    let mut p = Parser::new("@Dec({ option: true })\nclass Foo {}");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ClassDeclaration);

    let mut p = Parser::new("@A @B\nclass Foo {}");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    let class = match &node.data {
        NodeData::ClassDeclaration(d) => d,
        other => panic!("expected class, got {other:?}"),
    };
    let mods = class.modifiers.as_ref().unwrap();
    let decorators: Vec<_> = mods
        .iter()
        .filter(|n| n.kind == SyntaxKind::Decorator)
        .collect();
    assert_eq!(decorators.len(), 2);

    let mut p = Parser::new("@Namespace.Dec\nclass Foo {}");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ClassDeclaration);
}

#[test]
fn parse_regex_literal() {
    let mut p = Parser::new("let x = /foo/g;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::VariableStatement);

    let mut p = Parser::new("/foo/g.test(str);");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ExpressionStatement);

    let mut p = Parser::new("function f() { return /pattern/; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::FunctionDeclaration);

    let mut p = Parser::new(r"let x = /a\/b/;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new(r"let x = /[\/]/;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new("let x = a / b;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
}

#[test]
fn parse_regex_in_call_expression() {
    let mut p = Parser::new("let r = str.replace(/foo/g, 'bar');");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::VariableStatement);
}

#[test]
fn parse_comment_directives_propagate_to_source_file() {
    use crate::scanner::CommentDirectiveKind;
    let file = Parser::parse_source_file_text(
        "test.ts",
        "// @ts-ignore\nlet x = 1;\n// @ts-expect-error\n".to_string(),
    );
    assert_eq!(file.comment_directives.len(), 2);
    assert_eq!(
        file.comment_directives[0].kind,
        CommentDirectiveKind::Ignore
    );
    assert_eq!(
        file.comment_directives[1].kind,
        CommentDirectiveKind::ExpectError
    );
}

#[test]
fn parse_using_declaration() {
    let mut p = Parser::new("using x = getResource();");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::VariableStatement);

    let stmt = match &node.data {
        NodeData::VariableStatement(d) => d,
        other => panic!("expected variable statement, got {other:?}"),
    };
    assert!(stmt.declaration_list.flags.contains(NodeFlags::Using));

    let mut p = Parser::new("using = 1;");
    let node = p.parse_statement();
    assert_eq!(node.kind, SyntaxKind::ExpressionStatement);

    let mut p = Parser::new("using\nx = 1;");
    let node = p.parse_statement();
    assert_ne!(node.kind, SyntaxKind::VariableStatement);
}

#[test]
fn parse_await_using_declaration() {
    let mut p = Parser::new("await using x = getResource();");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::VariableStatement);
    let stmt = match &node.data {
        NodeData::VariableStatement(d) => d,
        other => panic!("expected variable statement, got {other:?}"),
    };
    assert!(stmt.declaration_list.flags.contains(NodeFlags::AwaitUsing));
}

#[test]
fn parse_accessor_property() {
    let mut p = Parser::new("class C { accessor x = 1; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ClassDeclaration);
    let class = match &node.data {
        NodeData::ClassDeclaration(d) => d,
        other => panic!("expected class, got {other:?}"),
    };
    let prop = &class.members.nodes[0];
    assert_eq!(prop.kind, SyntaxKind::PropertyDeclaration);
    let prop_data = match &prop.data {
        NodeData::PropertyDeclaration(d) => d,
        other => panic!("expected property, got {other:?}"),
    };
    let mods = prop_data.modifiers.as_ref().expect("expected modifiers");
    assert!(mods.modifier_flags.contains(ModifierFlags::Accessor));
}

#[test]
fn parse_type_predicate_in_function_type_return() {
    let mut p = Parser::new("type Predicate = (value: T) => value is S;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::TypeAliasDeclaration);

    let mut p = Parser::new("type P = (value: T, index: number) => value is S;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new("type P = () => this is T;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
}

#[test]
fn parse_type_predicate_in_method_return_type() {
    let mut p = Parser::new("interface I { isFoo(x: any): x is Foo; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::InterfaceDeclaration);

    let mut p = Parser::new("function isFoo(x: any): x is Foo { return true; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::FunctionDeclaration);

    let mut p = Parser::new("const isFoo = (x: any): x is Foo => true;");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
}

#[test]
fn parse_computed_property_name_in_type_member() {
    let mut p = Parser::new("interface I { [Symbol.iterator](): Iterator<T>; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::InterfaceDeclaration);

    let mut p = Parser::new("interface Symbol { [Symbol.toPrimitive](hint: string): symbol; }");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new("interface X { readonly [Symbol.toStringTag]: string; }");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new("interface X { [key: string]: number; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    let iface = match &node.data {
        NodeData::InterfaceDeclaration(d) => d,
        other => panic!("expected interface, got {other:?}"),
    };
    assert_eq!(iface.members.nodes[0].kind, SyntaxKind::IndexSignature);
}

#[test]
fn parse_contextual_keyword_as_property_name_in_type_member() {
    let mut p = Parser::new("interface X { readonly static: boolean; }");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new("interface X { readonly private: boolean; }");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p =
        Parser::new("interface EcdhKeyDeriveParams extends Algorithm { public: CryptoKey; }");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new("interface X { readonly x: boolean; }");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
}

#[test]
fn parse_heritage_clause_with_tuple_type_arguments() {
    let mut p = Parser::new("interface X extends Array<[number, number] | undefined> {}");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::InterfaceDeclaration);

    let mut p = Parser::new("interface X extends Foo<[number]> {}");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new("interface X extends A, Foo<[number, number]> {}");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new("class X extends Foo<[number, number]> {}");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
}

#[test]
fn parse_contextual_keyword_as_class_member_name() {
    let mut p = Parser::new("class C { static: number = 1; }");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new("class C { public: number = 1; }");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new("class C { readonly static: boolean; }");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new("class C { static x: number = 1; }");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
}

#[test]
fn parse_const_enum() {
    let mut p = Parser::new("const enum E { A, B, C }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::EnumDeclaration);
}

#[test]
fn parse_const_variable_not_treated_as_enum() {
    let mut p = Parser::new("const x = 1;");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::VariableStatement);
}

#[test]
fn parse_abstract_class() {
    let mut p = Parser::new("abstract class Animal { abstract makeSound(): void; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::ClassDeclaration);
}

#[test]
fn parse_async_function() {
    let mut p = Parser::new("async function fetchData(): Promise<void> { return; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::FunctionDeclaration);
}

#[test]
fn parse_async_generator() {
    let mut p = Parser::new("async function* gen() { yield 1; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::FunctionDeclaration);
}

#[test]
fn parse_yield_in_generator() {
    let mut p = Parser::new("function* counter() { yield 1; yield* [2, 3]; }");
    let node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
    assert_eq!(node.kind, SyntaxKind::FunctionDeclaration);
}

#[test]
fn parse_yield_await_in_async_generator() {
    let mut p = Parser::new("async function* gen() { yield await fetch('url'); }");
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
}

#[test]
fn parse_for_await_of() {
    let mut p = Parser::new(
        "async function process(stream) { for await (const chunk of stream) { console.log(chunk); } }",
    );
    let _node = p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
}

#[test]
fn parse_optional_chaining() {
    let mut p = Parser::new("const x = obj?.foo?.bar;");
    p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new("const x = obj?.foo?.();");
    p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
}

#[test]
fn parse_nullish_coalescing() {
    let mut p = Parser::new("const x = a ?? b;");
    p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
}

#[test]
fn parse_variance_annotations() {
    let mut p = Parser::new("interface Box<in T> { value: T; }");
    p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new("interface Box<out T> { value: T; }");
    p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);

    let mut p = Parser::new("interface Box<in out T> { value: T; }");
    p.parse_statement();
    assert!(p.diagnostics().is_empty(), "{:?}", p.diagnostics);
}
