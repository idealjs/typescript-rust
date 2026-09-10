use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_enum_as_namespace() {
    let content = r#"/*1*/enum /*2*/E { A }
let e: /*3*/E.A;"#;
    let mut s = Session::new_for_test("findAllRefsEnumAsNamespace", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3")
}
