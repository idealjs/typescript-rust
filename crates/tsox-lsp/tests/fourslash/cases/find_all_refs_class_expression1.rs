use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_class_expression1() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
module.exports = class /*0*/A {};
// @Filename: /b.js
import /*1*/A = require("./a");
/*2*/A;"#;
    let mut s = Session::new_for_test("findAllRefsClassExpression1", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "0", "1", "2")
}
