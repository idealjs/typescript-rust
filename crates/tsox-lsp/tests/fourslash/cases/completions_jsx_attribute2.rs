use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_jsx_attribute2() {
    let content = r#"// @jsx: preserve
// @Filename: /a.tsx
declare namespace JSX {
    interface Element {}
    interface IntrinsicElements {
        div: {
            /** Doc */
            foo: boolean;
            bar: string;
            "aria-foo": boolean;
        }
    }
}

<div foo /*1*/></div>;
<div foo={true} /*2*/></div>;
<div bar="test" /*3*/></div>;
<div aria-foo /*4*/></div>;"#;
    let mut s = Session::new_for_test("completionsJsxAttribute2", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["aria-foo", "bar"]);
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["aria-foo", "bar"]);
    fourslash::verify_completions_exact_at(&mut s, Some("3"), &["aria-foo", "foo"]);
    fourslash::verify_completions_exact_at(&mut s, Some("4"), &["bar", "foo"]);
}
