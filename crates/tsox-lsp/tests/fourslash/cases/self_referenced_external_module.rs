use tsox_lsp::fourslash::{self, Session};


#[test]
fn self_referenced_external_module() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: app.ts
export import A = require('./app');
export var I = 1;
A./**/I"#;
    let mut s = Session::new_for_test("selfReferencedExternalModule", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
