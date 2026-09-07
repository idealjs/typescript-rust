use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn references01() {
    let content = r#"// @lib: es5
// @Filename: /home/src/workspaces/project/referencesForGlobals_1.ts
class /*0*/globalClass {
    public f() { }
}
// @Filename: /home/src/workspaces/project/referencesForGlobals_2.ts
///<reference path="referencesForGlobals_1.ts" />
var c = /*1*/globalClass();"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
