use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_on_this() {
    let content = r#"interface Restricted {
    n: number;
}
function wrapper(wrapped: { (): void; }) { }
class Foo {
    n: number;
    prop1: th/*0*/is;
    public explicitThis(this: this) {
        wrapper(
            function explicitVoid(this: void) {
                console.log(th/*1*/is);
            }
        )
        console.log(th/*2*/is);
    }
    public explicitInterface(th/*3*/is: Restricted) {
        console.log(th/*4*/is);
    }
    public explicitClass(th/*5*/is: Foo) {
        console.log(th/*6*/is);
    }
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "0", "this", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "this: void", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "this: this", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(parameter) this: Restricted", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "this: Restricted", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "(parameter) this: Foo", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6", "this: Foo", "")
}
