use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_js_enum() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/** @enum {string} */
/*1*/const /*2*/E = { A: "" };
/*3*/E["A"];
/** @type {/*4*/E} */
const e = /*5*/E.A;"#;
    let _s = Session::new_for_test("findAllRefs_jsEnum", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5")
}
