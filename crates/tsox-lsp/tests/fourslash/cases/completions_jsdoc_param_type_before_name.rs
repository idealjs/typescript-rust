use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_jsdoc_param_type_before_name() {
    let content = r#"// @lib: es5
/** @param /*name1*/ {/*type*/} /*name2*/ */
function toString(obj) {}"#;
    let mut s = Session::new_for_test("completionsJsdocParamTypeBeforeName", content);
    fourslash::go_to_marker(&mut s, "type");
    // TODO: f.VerifyCompletions(t, "type", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("name1"), &["obj"]);
    fourslash::verify_completions_exact_at(&mut s, Some("name2"), &["obj"]);
}
