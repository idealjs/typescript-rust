use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: }"]
#[test]
fn edit_template_constraint() {
    let content = r#"/**
 * @template {/**/
 */
function f() {}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, "n");
    fourslash::insert(&mut s, "u");
    // TODO: }
}
