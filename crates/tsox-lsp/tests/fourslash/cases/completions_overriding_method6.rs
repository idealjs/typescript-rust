use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_overriding_method6() {
    let content = r#"// @Filename: a.ts
// @newline: LF
class Base {
    method() {}
    protected prop = 1;
}
class A extends Base {
    public abstract /*a*/
}
abstract class Ab extends Base {
    public abstract /*b*/
}
class B extends Base {
    public override [|m/*c*/|]
}
class C extends Base {
    override /*d*/
}
class f extends Base {
    protected /*f*/
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "c", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "d", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "f", &fourslash.CompletionsExpectedList{
}
