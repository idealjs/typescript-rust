use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_for_derived_type1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface IFoo {
    bar(): IFoo;
}
interface IFoo2 extends IFoo {
    bar2(): IFoo2;
}
var f: IFoo;
var f2: IFoo2;
f./*1*/; // completion here shows bar with return type is any
f2./*2*/ // here bar has return type any, but bar2 is Foo2"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
