use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn jsx_element_missing_opening_tag_no_crash() {
    let content = r#"//@Filename: file.tsx
declare function Foo(): any;
let x = <></Fo/*$*/o>;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "$", "let Foo: any", "")
}
