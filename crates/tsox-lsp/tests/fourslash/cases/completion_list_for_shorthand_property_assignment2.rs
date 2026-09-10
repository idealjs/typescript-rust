use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn completion_list_for_shorthand_property_assignment2() {
    let content = r#"var person: {name:string; id: number} = { n/**/"#;
    let mut s = Session::new_for_test("completionListForShorthandPropertyAssignment2", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["id", "name"]);
    // TODO: }
}
