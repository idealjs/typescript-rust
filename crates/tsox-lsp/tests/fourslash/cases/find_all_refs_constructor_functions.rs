use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_constructor_functions() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
function f() {
    /*1*/this./*2*/x = 0;
}
f.prototype.setX = function() {
    /*3*/this./*4*/x = 1;
}
f.prototype.useX = function() { this./*5*/x; }"#;
    let mut s = Session::new_for_test("findAllRefsConstructorFunctions", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5")
}
