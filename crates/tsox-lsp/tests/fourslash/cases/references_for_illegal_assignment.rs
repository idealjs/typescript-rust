use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_illegal_assignment() {
    let content = r#"f/*1*/oo = fo/*2*/o;
var /*bar*/bar = function () { };
bar = bar + 1;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "bar")
}
