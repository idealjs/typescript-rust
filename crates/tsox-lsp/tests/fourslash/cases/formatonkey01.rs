use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.Insert(t, '}')"]
#[test]
fn formatonkey01() {
    let content = r#"// @lib: es5
switch (1) {
    case 1:
        {
            /*1*/
        break;
}"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.Insert(t, "}")
}
