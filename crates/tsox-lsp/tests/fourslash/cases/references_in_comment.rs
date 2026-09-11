use tsox_lsp::fourslash::{self, Session};


#[test]
fn references_in_comment() {
    let content = r#"// References to /*1*/foo or b/*2*/ar
/* in comments should not find fo/*3*/o or bar/*4*/ */
class foo { }
var bar = 0;"#;
    let mut s = Session::new_for_test("referencesInComment", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
