use tsox_lsp::fourslash::Session;


#[test]
fn code_lens_function_expressions01() {
    let content = r#"
// @filename: anonymousFunctionExpressions.ts
export let anonFn1 = function () {};
export const anonFn2 = function () {};

let anonFn3 = function () {};
const anonFn4 = function () {};

// @filename: arrowFunctions.ts
export let arrowFn1 = () => {};
export const arrowFn2 = () => {};

let arrowFn3 = () => {};
const arrowFn4 = () => {};

// @filename: namedFunctions.ts
export let namedFn1 = function namedFn1() {
    namedFn1();
}
namedFn1();

export const namedFn2 = function namedFn2() {
    namedFn2();
}
namedFn2();

let namedFn3 = function namedFn3() {};
const namedFn4 = function namedFn4() {};
"#;
    let _s = Session::new_for_test("codeLensFunctionExpressions01", content);
    // TODO: f.VerifyBaselineCodeLens(t, &lsutil.UserPreferences{
}
