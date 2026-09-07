use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_merged_module() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strict: false
namespace M2 {
    export interface A {
        foo: string;
    }
    var a: A;
    var r = a.foo + a.bar;
}
namespace M2 {
    export interface A {
        bar: number;
    }
    var a: A;
    var r = a.fo/*1*/o + a.bar;
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) M2.A.foo: string", "");
    fourslash::verify_no_errors(&mut s);
}
