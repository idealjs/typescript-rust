use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_globals5() {
    let content = r#"// @Filename: referencesForGlobals_1.ts
namespace globalModule {
    export var x;
}

/*1*/import /*2*/globalAlias = globalModule;
// @Filename: referencesForGlobals_2.ts
var m = /*3*/globalAlias;"#;
    let mut s = Session::new_for_test("referencesForGlobals5", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
