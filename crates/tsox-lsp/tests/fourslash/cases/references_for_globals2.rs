use tsox_lsp::fourslash::Session;


#[test]
fn references_for_globals2() {
    let content = r#"// @Filename: referencesForGlobals_1.ts
/*1*/class /*2*/globalClass {
    public f() { }
}
// @Filename: referencesForGlobals_2.ts
var c = /*3*/globalClass();"#;
    let _s = Session::new_for_test("referencesForGlobals2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
