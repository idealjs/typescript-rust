use tsox_lsp::fourslash::Session;


#[test]
fn code_fix_forgotten_this_property_access04() {
    let content = r#"// @jsx: react
// @jsxFactory: factory
// @Filename: /a.tsx
export class C {
    foo() {
        return <a.div />;
    }
}"#;
    let _s = Session::new_for_test("codeFixForgottenThisPropertyAccess04", content);
    // TODO: f.VerifyCodeFixNotAvailable(t)
}
