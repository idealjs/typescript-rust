use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_jsx_namespaced_intrinsic() {
    let content = r#"// @jsx: react
// @Filename: /a.tsx
declare const React: any;
declare namespace JSX {
    interface Element {}
    interface IntrinsicElements {
        /** Element docs */
        "foo:bar": {
            /** Foo docs */
            foo: boolean
            /** Bar docs */
            bar: string
        }
    }
}
<foo:ba/*tag*/r fo/*attr*/o />"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "tag",
        "(property) JSX.IntrinsicElements[\"foo:bar\"]: {\n    foo: boolean;\n    bar: string;\n}",
        "Element docs",
    );
    fourslash::verify_quick_info_at(&mut s, "attr", "(property) foo: boolean", "Foo docs");
}
