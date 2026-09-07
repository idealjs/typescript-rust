use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) 'prop:foo': string", "This also appears")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(property) foo: string", "This appears")
}
