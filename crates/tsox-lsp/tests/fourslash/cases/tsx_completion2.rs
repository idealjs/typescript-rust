use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_completion2() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props; }
}
class MyComp { props: { ONE: string; TWO: number } }
var x = <MyComp /**//>;"#;
    let mut s = Session::new_for_test("tsxCompletion2", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["ONE", "TWO"]);
}
