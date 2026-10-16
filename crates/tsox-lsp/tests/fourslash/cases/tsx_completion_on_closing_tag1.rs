use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_completion_on_closing_tag1() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        div: { ONE: string; TWO: number; }
    }
}
var x1 = <div><//**/"#;
    let mut s = Session::new_for_test("tsxCompletionOnClosingTag1", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["div>"]);
}
