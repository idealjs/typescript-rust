use tsox_lsp::fourslash::{self, Session};


#[test]
fn hover_qualified_generic_names() {
    let content = r#"
function f<T>(x: T) {
    class C {
        value = x
    }
    return new C()
}

class A<T> {
    foo() {}
}
class B extends A<string> {}

let t1/*1*/ = f("hello")
const t2/*2*/ = new B()
t2./*3*/foo()
"#;
    let mut s = Session::new_for_test("hoverQualifiedGenericNames", content);
    fourslash::verify_quick_info_at(&mut s, "1", "let t1: f<string>.C", "");
    fourslash::verify_quick_info_at(&mut s, "2", "const t2: B", "");
    fourslash::verify_quick_info_at(&mut s, "3", "(method) A<string>.foo(): void", "");
}
