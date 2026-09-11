use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_overridden_member10() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @noEmit: true
// @noImplicitOverride: true
// @filename: a.js
class Foo {}
class Bar extends Foo {
    /** [|@override{|"name": "1"|} |]*/
    m() {}
}"#;
    let mut s = Session::new_for_test("goToDefinitionOverriddenMember10", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
