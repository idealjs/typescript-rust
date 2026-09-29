use tsox_lsp::fourslash::Session;


#[test]
fn go_to_implementation_satisfies() {
    let content = r#"// @filename: /a.ts
interface /*def*/I {
	foo: string;
}

function f() {
    const foo = { foo: '' } satisfies [|I|];
}"#;
    let _s = Session::new_for_test("goToImplementation_satisfies", content);
    // TODO: f.VerifyBaselineGoToImplementation(t, "def")
}
