use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn jsdoc_parameter_name_completion() {
    let content = r#"/**
 * @param /*0*/
 */
function f(foo, bar) {}
/**
 * @param foo
 * @param /*1*/
 */
function g(foo, bar) {}
/**
 * @param can/*2*/
 * @param cantaloupe
 */
function h(cat, canary, canoodle, cantaloupe, zebra) {}
/**
 * @param /*3*/ {string} /*4*/
 */
function i(foo, bar) {}"#;
    let mut s = Session::new_for_test("jsdocParameterNameCompletion", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"0", "3", "4"}, &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["bar"]);
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["canary", "canoodle"]);
}
