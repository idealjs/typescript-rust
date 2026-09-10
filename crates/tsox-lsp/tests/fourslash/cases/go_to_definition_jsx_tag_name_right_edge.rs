use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_jsx_tag_name_right_edge() {
    let content = r#"// @allowJs: true
// @jsx: react-jsx
// @filename: /a.jsx
export function Component() {
    return null;
}

export function App() {
    return <Component/*use*/ />
}"#;
    let mut s = Session::new_for_test("goToDefinitionJsxTagNameRightEdge", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "use")
}
