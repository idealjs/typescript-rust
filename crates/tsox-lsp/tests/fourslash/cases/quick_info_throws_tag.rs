use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineHover"]
#[test]
fn quick_info_throws_tag() {
    let content = r#"class E extends Error {}

/**
 * @throws {E}
 */
function f1() {}

/**
 * @throws {E} description
 */
function f2() {}

/**
 * @throws description
 */
function f3() {}
f1/*1*/()
f2/*2*/()
f3/*3*/()"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
    fourslash::unsupported("VerifyBaselineHover"); // f.VerifyBaselineHover(t)
}
