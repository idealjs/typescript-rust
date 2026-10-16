use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_jsx_namespaced_name_with_doc1() {
    let content = r#"// @jsx: react
// @Filename: /types.d.ts
declare namespace JSX {
  interface IntrinsicElements {
    'my-el': {
      /** This appears */
      foo: string;

      /** This also appears */
      'prop:foo': string;
    };
  }
}
// @filename: /a.tsx
<my-el /*1*/prop:foo="bar" /*2*/foo="baz" />"#;
    let mut s = Session::new_for_test("quickInfoOnJsxNamespacedNameWithDoc1", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) 'prop:foo': string", "This also appears");
    fourslash::verify_quick_info_at(&mut s, "2", "(property) foo: string", "This appears");
}
