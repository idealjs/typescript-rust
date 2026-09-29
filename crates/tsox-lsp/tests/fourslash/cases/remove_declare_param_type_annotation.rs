use tsox_lsp::fourslash::{self, Session};


#[test]
fn remove_declare_param_type_annotation() {
    let content = r#"declare class T { }
declare function parseInt(/**/s:T):T;
parseInt('2');"#;
    let mut s = Session::new_for_test("removeDeclareParamTypeAnnotation", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.DeleteAtCaret(t, 3)
}
