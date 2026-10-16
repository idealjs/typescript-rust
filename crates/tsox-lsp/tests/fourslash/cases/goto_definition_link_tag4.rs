use tsox_lsp::fourslash::Session;


#[test]
fn goto_definition_link_tag4() {
    let content = r#"// @filename: a.ts
interface [|/*2*/Foo|] {
    foo: E.Foo;
}
// @Filename: b.ts
enum E {
    /** {@link /*1*/[|Foo|]} */
    Foo
}"#;
    let _s = Session::new_for_test("gotoDefinitionLinkTag4", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "1")
}
