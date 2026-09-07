use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_comments_function_declaration() {
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
