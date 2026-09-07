use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_contextual_object_method_js_doc() {
    let content = r#"
interface I {
    /**
     * Description of func.
     * @param arg Description of arg.
     */
    func(arg: number): void
}

class Foo {
    constructor(i: I) {}
}

new Foo({ func/*1*/() {} })
"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(method) I.func(arg: number): void", "Description of func.\n\n*@param* 
}
