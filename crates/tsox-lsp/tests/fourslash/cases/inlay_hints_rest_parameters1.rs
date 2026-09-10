use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineInlayHints"]
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
    let mut s = Session::new_for_test("inlayHintsRestParameters1", content);
    fourslash::unsupported("VerifyBaselineInlayHints"); // f.VerifyBaselineInlayHints(t, nil /*span*/, &lsutil.UserPreferences{InlayHints: lsutil.InlayHintsPre
}
