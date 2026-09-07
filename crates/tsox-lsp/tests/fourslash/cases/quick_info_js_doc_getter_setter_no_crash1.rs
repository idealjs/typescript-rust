use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_js_doc_getter_setter_no_crash1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"class A implements A {
  get x(): string { return "" }
}
const e = new A()
e.x/*1*/

class B implements B {
  set x(v: string) {}
}
const f = new B()
f.x/*2*/"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) A.x: string", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(property) B.x: string", "");
}
