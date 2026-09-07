use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_getter_setter() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @target: es2015
class C {
    #x = Promise.resolve("")
    set /*setterDef*/myValue(x: Promise<string> | string) {
        this.#x = Promise.resolve(x);
    }
    get /*getterDef*/myValue(): Promise<string> {
        return this.#x;
    }
}
let instance = new C();
instance./*setterUse*/myValue = instance./*getterUse*/myValue;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "getterUse", "(property) C.myValue: Promise<string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "getterDef", "(getter) C.myValue: Promise<string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "setterUse", "(property) C.myValue: string | Promise<string>", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "setterDef", "(setter) C.myValue: string | Promise<string>", "")
}
