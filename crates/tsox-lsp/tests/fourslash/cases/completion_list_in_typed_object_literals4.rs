use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_typed_object_literals4() {
    let content = r#"interface MyPoint {
    x1: number;
    y1: number;
}
var p15: MyPoint = {
    "x1": 5,
    /*15*/
};"#;
    let mut s = Session::new_for_test("completionListInTypedObjectLiterals4", content);
    fourslash::verify_completions_exact_at(&mut s, Some("15"), &["y1"]);
}
