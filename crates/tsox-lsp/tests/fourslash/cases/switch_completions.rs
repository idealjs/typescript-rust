use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn switch_completions() {
    let content = r#"enum E { A, B }
declare const e: E;
switch (e) {
    case E.A:
        return 0;
    case E./*1*/
}
declare const f: 1 | 2 | 3;
switch (f) {
    case 1:
        return 1;
    case /*2*/
}
declare const f2: 'foo' | 'bar' | 'baz';
switch (f2) {
    case 'bar':
        return 1;
    case '/*3*/'
}

// repro from #52874
declare let x: "foo" | "bar";
switch (x) {
    case ('/*4*/')
}"#;
    let mut s = Session::new_for_test("switchCompletions", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_include_exclude_at(&mut s, Some("4"), &["foo", "bar"], &[]);
}
