use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_js_enum() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/** @enum {string} */
/*1*/const /*2*/E = { A: "" };
/*3*/E["A"];
/** @type {/*4*/E} */
const e = /*5*/E.A;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5")
}
