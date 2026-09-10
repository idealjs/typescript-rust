use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn type_reference_on_server() {
    let content = r#"// @lib: es5
/// <reference types="foo" />
var x: number;
x./*1*/"#;
    let mut s = Session::new_for_test("typeReferenceOnServer", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::verify_completions_include_exclude_at(&mut s, Some("1"), &["toFixed"], &[]);
}
