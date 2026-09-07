use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_list_protected_members() {
    let content = r#"class Base {
    protected y;
    constructor(protected x) {}
    method() { this./*1*/; }
}
class D1 extends Base {
    protected z;
    method1() { this./*2*/; }
}
class D2 extends Base {
    method2() { this./*3*/; }
}
class D3 extends D1 {
    method2() { this./*4*/; }
}
var b: Base;
f./*5*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", nil)
}
