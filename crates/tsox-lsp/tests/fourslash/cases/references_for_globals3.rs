use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_globals3() {
    let content = r#"// @Filename: referencesForGlobals_1.ts
/*1*/interface /*2*/globalInterface {
     f();
}
// @Filename: referencesForGlobals_2.ts
var i: /*3*/globalInterface;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
