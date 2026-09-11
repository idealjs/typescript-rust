use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_module_dot_exports() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/*1*/const b = require("/*2*/./b");
// @Filename: /b.js
/*3*/module.exports = 0;"#;
    let mut s = Session::new_for_test("findAllRefsModuleDotExports", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
