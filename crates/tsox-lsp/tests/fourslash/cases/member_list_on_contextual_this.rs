use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn member_list_on_contextual_this() {
    let content = r#"interface A {
    a: string;
}
declare function ctx(callback: (this: A) => string): string;
ctx(function () { return th/*1*/is./*2*/a });"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "this: A", "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
