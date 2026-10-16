use tsox_lsp::fourslash::{self, Session};


#[test]
fn delete_extension_in_reopened_interface() {
    let content = r#"interface A { a: number; }
interface B { b: number; }

interface I /*del*/extends A { }
interface I extends B { }

var i: I;
class C /*delImplements*/implements A { }
var c: C;
c.a;"#;
    let mut s = Session::new_for_test("deleteExtensionInReopenedInterface", content);
    fourslash::go_to_marker(&mut s, "del");
    // TODO: f.DeleteAtCaret(t, 9)
    fourslash::go_to_eof(&mut s, );
    fourslash::insert(&mut s, "var a = i.a;");
    fourslash::go_to_marker(&mut s, "delImplements");
    // TODO: f.DeleteAtCaret(t, 12)
    fourslash::go_to_marker(&mut s, "del");
    fourslash::insert(&mut s, "extends A");
}
