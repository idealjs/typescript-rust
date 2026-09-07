use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
#[test]
fn inlay_hints_interactive_parameter_names_with_comments() {
    let content = r#"const fn = (x: any) => { }
fn(/* nobody knows exactly what this param is */ 42);
function foo (aParameter: number, bParameter: number, cParameter: number) { }
foo(
    /** aParameter */
    1,
    // bParameter
    2,
    /* cParameter */
    3
)
foo(
    /** multiple comments */
    /** aParameter */
    1,
    /** bParameter */
    /** multiple comments */
    2,
    // cParameter
    /** multiple comments */
    3
)
foo(
    /** wrong name */
    1,
    2,
    /** multiple */
    /** wrong */
    /** name */
    3
)"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
