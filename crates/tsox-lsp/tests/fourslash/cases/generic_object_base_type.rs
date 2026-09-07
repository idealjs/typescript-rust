use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn generic_object_base_type() {
    let content = r#"// @strict: false
class C<T> {
    constructor(){}
    foo(a: T) {
        return a.toString();
    }
}
var x = new C<string>();
var y: string = x.foo("hi");
/*1*/"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
