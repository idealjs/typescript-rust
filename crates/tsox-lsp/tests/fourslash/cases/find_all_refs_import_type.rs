use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_import_type() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
module.exports = 0;
/*1*/export type /*2*/N = number;
// @Filename: /b.js
type T = import("./a")./*3*/N;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
