use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_for_globals3() {
    let content = r#"// @Filename: referencesForGlobals_1.ts
/*1*/interface /*2*/globalInterface {
     f();
}
// @Filename: referencesForGlobals_2.ts
var i: /*3*/globalInterface;"#;
    let mut s = Session::new_for_test("referencesForGlobals3", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
