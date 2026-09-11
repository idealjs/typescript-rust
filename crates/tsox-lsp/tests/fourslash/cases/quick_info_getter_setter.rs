use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_getter_setter() {
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
    let mut s = Session::new_for_test("quickInfoGetterSetter", content);
    fourslash::verify_quick_info_at(&mut s, "getterUse", "(property) C.myValue: Promise<string>", "");
    fourslash::verify_quick_info_at(&mut s, "getterDef", "(getter) C.myValue: Promise<string>", "");
    fourslash::verify_quick_info_at(&mut s, "setterUse", "(property) C.myValue: string | Promise<string>", "");
    fourslash::verify_quick_info_at(&mut s, "setterDef", "(setter) C.myValue: string | Promise<string>", "");
}
