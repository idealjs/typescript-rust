use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn fundule_with_recursive_reference() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"namespace M {
    export function C() {}
    export namespace C {
    export var /**/C = M.C
  }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var M.C.C: typeof M.C", "")
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
