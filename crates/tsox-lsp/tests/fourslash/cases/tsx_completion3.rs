use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_completion3() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        div: { one; two; }
    }
}
<div one={1} /**//>;"#;
    let mut s = Session::new_for_test("tsxCompletion3", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["two"]);
}
