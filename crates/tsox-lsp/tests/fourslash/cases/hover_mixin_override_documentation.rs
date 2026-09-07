use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn hover_mixin_override_documentation() {
    let content = r#"
// @strict: true
// @filename: main.ts

declare class BaseClass {
    /** some documentation */
    static method(): number;
}

type AnyConstructor = abstract new (...args: any[]) => object

class MixinClass {}
declare function Mix<T extends AnyConstructor>(BaseClass: T): typeof MixinClass & T;

declare class Mixed extends Mix(BaseClass) {
    static method(): number;
}

Mixed./*1*/method;
"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(method) Mixed.method(): number", "some documentation")
}
