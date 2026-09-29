use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_in_typed_object_literals_with_partial_property_names() {
    let content = r#"interface MyPoint {
    x1: number;
    y1: number;
}
var p15: MyPoint = {
    /**/
};"#;
    let mut s = Session::new_for_test("completionListInTypedObjectLiteralsWithPartialPropertyNames", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["x1", "y1"]);
    fourslash::insert(&mut s, "x");
    fourslash::verify_completions_exact_at(&mut s, None, &["x1", "y1"]);
    fourslash::insert(&mut s, "1");
    fourslash::verify_completions_exact_at(&mut s, None, &["x1", "y1"]);
    fourslash::insert(&mut s, ": null,");
    fourslash::verify_completions_exact_at(&mut s, None, &["y1"]);
}
