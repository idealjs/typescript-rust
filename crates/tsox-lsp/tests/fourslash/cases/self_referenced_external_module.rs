use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn self_referenced_external_module() {
    let content = r#"// @Filename: app.ts
export import A = require('./app');
export var I = 1;
A./**/I"#;
    let mut s = Session::new_for_test("selfReferencedExternalModule", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
