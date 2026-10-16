use tsox_lsp::fourslash::Session;


#[test]
fn find_all_refs_for_function_expression01() {
    let content = r#"// @Filename: file1.ts
var foo = /*1*/function /*2*/foo(a = /*3*/foo(), b = () => /*4*/foo) {
    /*5*/foo(/*6*/foo, /*7*/foo);
}
// @Filename: file2.ts
/// <reference path="file1.ts" />
foo();"#;
    let _s = Session::new_for_test("findAllRefsForFunctionExpression01", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6", "7")
}
