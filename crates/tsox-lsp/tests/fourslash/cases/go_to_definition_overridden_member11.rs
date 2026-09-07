use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_overridden_member11() {
    let content = r#"// @allowJs: true
// @checkJs: true
// @noEmit: true
// @noImplicitOverride: true
// @filename: a.js
class Foo {
    /*Foo_m*/m() {}
}
class Bar extends Foo {
    /** @[|over{|"name": "1"|}ride|][| se{|"name": "2"|}e {@li{|"name": "3"|}nk https://test.c{|"name": "4"|}om} {|"name": "5"|}description |]*/
    m() {}
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1", "2", "3", "4", "5")
}
