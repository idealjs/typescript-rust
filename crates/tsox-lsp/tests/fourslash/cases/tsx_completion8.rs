use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn tsx_completion8() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        div: { ONE: string; TWO: number; }
    }
}
var x = <div /*1*/ autoComplete /*2*/ />;"#;
    let mut s = Session::new_for_test("tsxCompletion8", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}
