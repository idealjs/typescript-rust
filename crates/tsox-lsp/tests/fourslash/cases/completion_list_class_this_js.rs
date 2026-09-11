use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_class_this_js() {
    let content = r#"// @Filename: completionListClassThisJS.js
// @allowJs: true
/** @typedef {number} CallbackContext */
class Foo {
    bar() {
       this/**/
    }
    /** @param {function (this: CallbackContext): any} cb */
    baz(cb) {
    }
}"#;
    let mut s = Session::new_for_test("completionListClassThisJS", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
