use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn rename_js_exports03() {
    let content = r#"// @allowJs: true
// @Filename: a.js
class /*1*/A {
    /*2*/constructor() { }
}
module.exports = A;
// @Filename: b.js
const /*3*/A = require("./a");
new /*4*/A;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
