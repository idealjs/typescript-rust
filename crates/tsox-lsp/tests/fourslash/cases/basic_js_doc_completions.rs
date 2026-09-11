use tsox_lsp::fourslash::{self, Session};


#[test]
fn basic_js_doc_completions() {
    let content = r#"
// @filename: file.js
// @allowJs: true
/**
 * @/*1*/
 */
function foo(x) {
  return x + 1;
}
  
/**
 * /*2*/
 */
function bar(x, { y }) {
  return x + y;
}

/**
 * @param {number} x
 * /*3*/
 */
function baz(x, { y }) {
  return x + y;
}

/**
 * @param {number} x
 * @param {object} param1 
 * @param {n/*4*/} param1.y 
 */
function baz(x, { y }) {
  return x + y;
}

/**
 * @/*5*/
 */
function baz(x = 0) {
  return x * 2;
}
"#;
    let mut s = Session::new_for_test("basicJSDocCompletions", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_include_exclude_at(&mut s, Some("2"), &["@param", "@param {*} x "], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("3"), &["@param", "@param {object} param1 \\n* @param {*} param1.y "], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("4"), &["number"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("5"), &["param {number} [x=0] "], &[]);
}
