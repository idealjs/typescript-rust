use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_typedef_tag() {
    let content = r#"// @allowJs: true
// @Filename: a.js
/**
 * The typedef tag should not appear in the quickinfo.
 * @typedef {{ foo: 'foo' }} Foo
 */
function f() { }
f/*1*/()
/**
 * A removed comment
 * @tag Usage shows that non-param tags in comments explain the typedef instead of using it
 * @typedef {{ nope: any }} Nope not here
 * @tag comment 2
 */
function g() { }
g/*2*/()
/**
 * The whole thing is kept
 * @param {Local} keep
 * @typedef {{ local: any }} Local kept too
 * @returns {void} also kept
 */
function h(keep) { }
h/*3*/({ nope: 1 })"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
