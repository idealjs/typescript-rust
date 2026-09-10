use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_completion6() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        div: { ONE: string; TWO: number; }
    }
}
var x = <div ONE='hello' /**/ />;"#;
    let mut s = Session::new_for_test("tsxCompletion6", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["TWO"]);
}
