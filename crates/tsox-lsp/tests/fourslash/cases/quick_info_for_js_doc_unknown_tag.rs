use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_for_js_doc_unknown_tag() {
    let content = r#"/**
 * @example
 * if (true) {
 *     foo()
 * }
 */
function fo/*1*/o() {
    return '2';
}
/**
 @example
 {
     foo()
 }
 */
function fo/*2*/o2() {
    return '2';
}
/**
 * @example
 *   x y
 *   12345
 *      b
 */
function m/*3*/oo() {
    return '2';
}
/**
 * @func
 * @example
 *   x y
 *   12345
 *      b
 */
function b/*4*/oo() {
    return '2';
}
/**
 * @func
 * @example    x y
 *             12345
 *                b
 */
function go/*5*/o() {
    return '2';
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
