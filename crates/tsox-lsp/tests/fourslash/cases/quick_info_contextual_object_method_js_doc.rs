use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("quickInfoContextualObjectMethodJSDoc", content);
    // TODO: f.VerifyQuickInfoAt(t, "1", "(method) I.func(arg: number): void", "Description of func.\n\n*@param* 
}
