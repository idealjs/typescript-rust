use tsox_lsp::fourslash::{self, Session};


#[test]
fn process_invalid_syntax1() {
    let content = r#"// @allowJs: true
// @Filename: decl.js
var obj = {};
// @Filename: unicode1.js
obj.𝒜 ;
// @Filename: unicode2.js
obj.¬ ;
// @Filename: unicode3.js
obj¬
// @Filename: forof.js
for (obj/**/.prop of arr) {

}"#;
    let mut s = Session::new_for_test("processInvalidSyntax1", content);
    // TODO: f.VerifyBaselineRename(t, nil /*preferences*/, "")
}
