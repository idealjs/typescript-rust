use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn references_in_configured_project() {
    let content = r#"// @Filename: /home/src/workspaces/project/referencesForGlobals_1.ts
class /*0*/globalClass {
    public f() { }
}
// @Filename: /home/src/workspaces/project/referencesForGlobals_2.ts
var c = /*1*/globalClass();
// @Filename: /home/src/workspaces/project/tsconfig.json
{ "files": ["referencesForGlobals_1.ts", "referencesForGlobals_2.ts"], "compilerOptions": { "lib": ["es5"] } }"#;
    let mut s = Session::new_for_test("referencesInConfiguredProject", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
