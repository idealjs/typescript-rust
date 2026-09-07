use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineSignatureHelp"]
#[test]
fn signature_help_comments_function_declaration_vs() {
    let content = r#"/** This comment should appear for foo*/
function foo() {
}
foo(/*4*/);
/** This is comment for function signature*/
function fooWithParameters(/** this is comment about a*/a: string,
    /** this is comment for b*/
    b: number) {
    var d = a;
}
fooWithParameters(/*10*/"a",/*11*/10);
/**
* Does something
* @param a a string
*/
declare function fn(a: string);
fn(/*12*/"hello");"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyBaselineSignatureHelp"); // f.VerifyBaselineSignatureHelp(t)
}
