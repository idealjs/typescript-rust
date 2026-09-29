use tsox_lsp::fourslash::{self, Session};


#[test]
fn recursive_generics2() {
    let content = r#"class S18<B, B, A, B> extends S18<A[], { S19: A; (): A }[]> { }
/**/"#;
    let mut s = Session::new_for_test("recursiveGenerics2", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "(new S18()).S18 = 0;");
}
