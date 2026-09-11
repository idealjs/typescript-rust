use tsox_lsp::fourslash::{self, Session};


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
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "use")
}
