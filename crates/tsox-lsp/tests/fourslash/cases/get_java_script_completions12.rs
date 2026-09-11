use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn get_java_script_completions12() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
/**
 * @param {number} input
 * @param {string} currency
 * @returns {number}
 */
var convert = function(input, currency) {
    switch(currency./*1*/) {
            case "USD":
            input./*2*/;
            case "EUR":
                return "" + rateToUsd.EUR;
            case "CNY":
                return {} + rateToUsd.CNY;
    }
}
convert(1, "")./*3*/
/**
 * @param {number} x
 */
var test1 = function(x) { return x./*4*/ }, test2 = function(a) { return a./*5*/ };"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions12", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"2", "3", "4"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
}
