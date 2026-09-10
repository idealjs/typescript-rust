use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn goto_definition_link_tag3() {
    let content = r#"// @Filename: /a.ts
enum E {
    /** {@link /*1*/[|Foo|]} */
    Foo
}
interface [|/*2*/Foo|] {
    foo: E.Foo;
}"#;
    let mut s = Session::new_for_test("gotoDefinitionLinkTag3", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, false, "1")
}
