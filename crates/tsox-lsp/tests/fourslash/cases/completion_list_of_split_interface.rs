use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_of_split_interface() {
    let content = r#"interface A {
    a: number;
}
interface I extends A {
    i1: number;
}
interface I1 extends A {
    i11: number;
}
interface B {
    b: number;
}
interface B1 {
    b1: number;
}
interface I extends B {
    i2: number;
}
interface I1 extends B, B1 {
    i12: number;
}
interface C {
    c: number;
}
interface I extends C {
    i3: number;
}
var ci: I;
ci./*1*/b;
var ci1: I1;
ci1./*2*/b;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
