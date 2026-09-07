use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn member_list_on_explicit_this() {
    let content = r#"interface Restricted {
   n: number;
}
class C1 implements Restricted {
   n: number;
   m: number;
   f(this: this) {this./*1*/} // test on 'this.'
   g(this: Restricted) {this./*2*/}
}
function f(this: void) {this./*3*/}
function g(this: Restricted) {this./*4*/}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"2", "4"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", nil)
}
