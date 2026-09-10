use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineVSHover"]
#[test]
fn quick_info_comments_function_declaration_vs() {
    let content = r#"/** This comment should appear for foo*/
function f/*1*/oo() {
}
f/*2*/oo();
/** This is comment for function signature*/
function fo/*5*/oWithParameters(/** this is comment about a*/a: string,
    /** this is comment for b*/
    b: number) {
    var /*6*/d = a;
}
fooWithParam/*8*/eters("a",10);
/**
* Does something
* @param a a string
*/
declare function fn(a: string);
fn("hello");"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyBaselineVSHover"); // f.VerifyBaselineVSHover(t)
}
