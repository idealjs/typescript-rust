use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixNotAvailable"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixNotAvailable"); // f.VerifyCodeFixNotAvailable(t)
}
