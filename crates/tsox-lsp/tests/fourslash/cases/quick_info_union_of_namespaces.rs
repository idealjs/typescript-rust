use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_union_of_namespaces() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"declare const x: typeof A | typeof B;
x./**/f;

namespace A {
    export function f() {}
}
namespace B {
    export function f() {}
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "", "(method) f(): void", "");
}
