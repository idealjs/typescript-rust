use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_satisfies() {
    let content = r#"// @filename: /a.ts
interface /*def*/I {
	foo: string;
}

function f() {
    const foo = { foo: '' } satisfies [|I|];
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "def")
}
