use tsox_lsp::fourslash::Session;


#[test]
fn inlay_hints_rest_parameters1() {
    let content = r#"function foo1(a: number, ...b: number[]) {}
foo1(1, 1, 1, 1);
type Args2 = [a: number, b: number]
declare function foo2(c: number, ...args: Args2);
foo2(1, 2, 3)
type Args3 = [number, number]
declare function foo3(c: number, ...args: Args3);
foo3(1, 2, 3)"#;
    let _s = Session::new_for_test("inlayHintsRestParameters1", content);
    // TODO: f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
