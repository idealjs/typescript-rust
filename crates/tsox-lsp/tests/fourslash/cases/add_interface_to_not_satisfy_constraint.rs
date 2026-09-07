use tsox_lsp::fourslash::{self, Session};

#[test]
fn add_interface_to_not_satisfy_constraint() {
    let content = r#"interface A {
	a: number;
}
/**/
interface C<T extends A> {
    x: T;
}

var v2: C<B>; // should not work"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "interface B { b: string; }");
}
