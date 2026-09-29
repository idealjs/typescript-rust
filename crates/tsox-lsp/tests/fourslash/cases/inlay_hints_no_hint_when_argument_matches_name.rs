use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_no_hint_when_argument_matches_name() {
    let content = r#"function foo (a: number, b: number) {}
declare const a: 1;
foo(a, 2);
declare const v: any;
foo(v.a, v.a);
foo(v.b, v.b);
foo(v.c, v.c);"#;
    let _s = Session::new_for_test("inlayHintsNoHintWhenArgumentMatchesName", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
