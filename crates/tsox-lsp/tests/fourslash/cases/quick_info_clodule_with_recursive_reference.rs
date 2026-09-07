use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_clodule_with_recursive_reference() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"namespace M {
    export class C {
        foo() { }
    }
    export namespace C {
    export var /**/C = M.C
  }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "", "var M.C.C: typeof M.C", "")
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
