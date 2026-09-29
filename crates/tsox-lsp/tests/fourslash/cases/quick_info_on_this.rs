use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("quickInfoOnThis", content);
    fourslash::verify_quick_info_at(&mut s, "0", "this", "");
    fourslash::verify_quick_info_at(&mut s, "1", "this: void", "");
    fourslash::verify_quick_info_at(&mut s, "2", "this: this", "");
    fourslash::verify_quick_info_at(&mut s, "3", "(parameter) this: Restricted", "");
    fourslash::verify_quick_info_at(&mut s, "4", "this: Restricted", "");
    fourslash::verify_quick_info_at(&mut s, "5", "(parameter) this: Foo", "");
    fourslash::verify_quick_info_at(&mut s, "6", "this: Foo", "");
}
