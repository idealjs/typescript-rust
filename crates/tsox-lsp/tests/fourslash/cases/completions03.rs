use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions03() {
    let content = r#"// @lib: es5
interface Foo {
   one: any;
   two: any;
   three: any;
}

let x: Foo = {
    get one() { return "" },
    set two(t) {},
    /**/
}"#;
    let mut s = Session::new_for_test("completions03", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["three"]);
}
