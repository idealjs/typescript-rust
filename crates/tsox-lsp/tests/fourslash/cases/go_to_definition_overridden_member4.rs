use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_overridden_member4() {
    let content = r#"// @noImplicitOverride: true
class Foo {
    /*2*/m() {}
}
function f () {
    return class extends Foo {
        [|/*1*/override|] m() {}
    }
}"#;
    let mut s = Session::new_for_test("goToDefinitionOverriddenMember4", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "1")
}
