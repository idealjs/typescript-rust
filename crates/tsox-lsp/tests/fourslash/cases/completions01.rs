use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions01() {
    let content = r#"// @lib: es5
var x: string[] = [];
x.forEach(function (y) { y/*1*/
x.forEach(y => y/*2*/"#;
    let mut s = Session::new_for_test("completions01", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, ".");
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["trim"], &[]);
    fourslash::insert(&mut s, "});");
    fourslash::go_to_marker(&mut s, "2");
    fourslash::insert(&mut s, ".");
    fourslash::verify_completions_include_exclude_at(&mut s, None, &["trim"], &[]);
}
