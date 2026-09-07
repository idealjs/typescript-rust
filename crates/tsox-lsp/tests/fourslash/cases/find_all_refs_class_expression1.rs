use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_class_expression1() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
module.exports = class /*0*/A {};
// @Filename: /b.js
import /*1*/A = require("./a");
/*2*/A;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "0", "1", "2")
}
