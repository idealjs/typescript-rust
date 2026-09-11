use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_link5() {
    let content = r#"function g() { }
/**
 * {@link g()} {@link g() } {@link g ()} {@link g () 0} {@link g()1} {@link g() 2}
 * {@link u()} {@link u() } {@link u ()} {@link u () 0} {@link u()1} {@link u() 2}
 */
function f(x) {
}
f/*3*/()"#;
    let mut s = Session::new_for_test("jsdocLink5", content);
    // TODO: f.VerifyBaselineHover(t)
}
