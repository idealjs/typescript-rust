use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("quickInfoThrowsTag", content);
    fourslash::verify_no_errors(&mut s, );
    // TODO: f.VerifyBaselineHover(t)
}
