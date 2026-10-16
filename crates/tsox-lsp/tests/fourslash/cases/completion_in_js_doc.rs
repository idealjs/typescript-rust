use tsox_lsp::fourslash::Session;


#[test]
fn completion_in_js_doc() {
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/** @/*1*/ */
var v1;

/** @p/*2*/ */
var v2;

/** @param /*3*/ */
var v3;

/** @param { n/*4*/ } bar */
var v4;

/** @type { n/*5*/ } */
var v5;

// @/*6*/
var v6;

// @pa/*7*/
var v7;

/** @return { n/*8*/ } */
var v8;

/** /*9*/ */

/**
 /*10*/
*/

/**
 * /*11*/
 */

/**
          /*12*/
 */

/**
  *       /*13*/
  */

/**
  * some comment /*14*/
  */

/**
  * @param /*15*/
  */

/** @param /*16*/ */

/**
  * jsdoc inline tag {@/*17*/}
  */"#;
    let _s = Session::new_for_test("completionInJsDoc", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"3", "15", "16"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"4", "5", "8"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"6", "7", "14"}, nil)
    // TODO: f.VerifyCompletions(t, []string{"9", "10", "11", "12", "13"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"17"}, &fourslash.CompletionsExpectedList{
}
