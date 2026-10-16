use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_for_shorthand_property_assignment() {
    let content = r#"var person: {name:string; id: number} = { n/**/"#;
    let mut s = Session::new_for_test("completionListForShorthandPropertyAssignment", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["id", "name"]);
    // TODO: }
}
