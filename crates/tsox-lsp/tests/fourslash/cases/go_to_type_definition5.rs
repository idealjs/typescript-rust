use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_type_definition5() {
    let content = r#"// @Filename: foo.ts
let Foo: /*definition*/unresolved;
type Foo = { x: string };
/*reference*/Foo;"#;
    let mut s = Session::new_for_test("goToTypeDefinition5", content);
    // TODO: f.VerifyBaselineGoToTypeDefinition(t, "reference")
}
