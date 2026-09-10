use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_string_literal_types() {
    let content = r#"type Options = "/*1*/option 1" | "option 2";
let myOption: Options = "/*2*/option 1";"#;
    let mut s = Session::new_for_test("findAllRefsForStringLiteralTypes", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2")
}
