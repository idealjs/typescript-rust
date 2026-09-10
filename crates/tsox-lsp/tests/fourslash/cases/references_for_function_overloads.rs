use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn references_for_function_overloads() {
    let content = r#"/*1*/function /*2*/foo(x: string);
/*3*/function /*4*/foo(x: string, y: number) {
    /*5*/foo('', 43);
}"#;
    let mut s = Session::new_for_test("referencesForFunctionOverloads", content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5")
}
