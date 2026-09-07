use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_for_function_expression01() {
    let content = r#"// @Filename: file1.ts
var foo = /*1*/function /*2*/foo(a = /*3*/foo(), b = () => /*4*/foo) {
    /*5*/foo(/*6*/foo, /*7*/foo);
}
// @Filename: file2.ts
/// <reference path="file1.ts" />
foo();"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1", "2", "3", "4", "5", "6", "7")
}
