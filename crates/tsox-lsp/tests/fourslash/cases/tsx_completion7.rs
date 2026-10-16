use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_completion7() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        div: { ONE: string; TWO: number; }
    }
}
let y = { ONE: '' };
var x = <div {...y} /**/ />;"#;
    let mut s = Session::new_for_test("tsxCompletion7", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
