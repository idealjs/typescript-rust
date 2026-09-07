use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
}
