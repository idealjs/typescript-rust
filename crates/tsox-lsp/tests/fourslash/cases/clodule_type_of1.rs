use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn clodule_type_of1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: false
class C<T> {
    static foo(x: number) { }
    x: T;
}

namespace C {
    export function f(x: typeof C) {
        x./*1*/
        var /*3*/r = new /*2*/x<number>();
        var /*5*/r2 = r./*4*/
        return typeof r;
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "foo(1);");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(local var) r: C<number>", "")
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "x;");
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "(local var) r2: number", "")
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
