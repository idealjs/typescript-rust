use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_type_parameter_in_merged_interface() {
    let content = r#"interface I</*1*/T> { a: /*2*/T }
interface I</*3*/T> { b: /*4*/T }"#;
    let mut s = Session::new_for_test("findAllRefsTypeParameterInMergedInterface", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4")
}
