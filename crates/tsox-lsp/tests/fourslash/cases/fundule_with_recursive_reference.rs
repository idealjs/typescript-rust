use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn fundule_with_recursive_reference() {
    let content = r#"namespace M {
    export function C() {}
    export namespace C {
    export var /**/C = M.C
  }
}"#;
    let mut s = Session::new_for_test("funduleWithRecursiveReference", content);
    fourslash::verify_quick_info_at(&mut s, "", "var M.C.C: typeof M.C", "");
    fourslash::verify_no_errors(&mut s, );
}
