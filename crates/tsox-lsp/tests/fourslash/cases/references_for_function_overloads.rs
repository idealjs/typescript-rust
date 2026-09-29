use tsox_lsp::fourslash::Session;


#[test]
fn references_for_function_overloads() {
    let content = r#"/*1*/function /*2*/foo(x: string);
/*3*/function /*4*/foo(x: string, y: number) {
    /*5*/foo('', 43);
}"#;
    let _s = Session::new_for_test("referencesForFunctionOverloads", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5")
}
