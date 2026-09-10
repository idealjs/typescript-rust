use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn tsx_completion4() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
        div: { one; two; }
    }
}
let bag = { x: 100, y: 200 };
<div {.../**/"#;
    let mut s = Session::new_for_test("tsxCompletion4", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["bag"], &[]);
    // TODO: }
}
