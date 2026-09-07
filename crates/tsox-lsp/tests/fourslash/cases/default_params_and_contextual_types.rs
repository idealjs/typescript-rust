use tsox_lsp::fourslash::{self, Session};

#[test]
fn default_params_and_contextual_types() {
    let content = r#"// @strict: false
interface FooOptions {
    text?: string;
}
interface Foo {
    bar(xy: string, options?: FooOptions): void;
}
var o: Foo = {
    bar: function (x/*1*/y, opt/*2*/ions = {}) {
        // expect xy to have type string, and options to have type FooOptions in here
    }
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "(parameter) xy: string", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(parameter) options: FooOptions", "");
}
