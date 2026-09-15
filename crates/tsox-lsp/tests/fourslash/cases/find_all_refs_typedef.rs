use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_typedef() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/**
 * @typedef I {Object}
 * /*1*/@prop /*2*/p {number}
 */

/** @type {I} */
let x;
x./*3*/p;"#;
    let _s = Session::new_for_test("findAllRefsTypedef", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
