use tsox_lsp::fourslash::{self, Session};


#[test]
fn formatonkey01() {
    let content = r#"// @lib: es5
switch (1) {
    case 1:
        {
            /*1*/
        break;
}"#;
    let mut s = Session::new_for_test("formatonkey01", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "1");
    fourslash::insert(&mut s, "}");
    // TODO: f.VerifyCurrentLineContent(t, `
}
