use tsox_lsp::fourslash::{self, Session};


#[test]
fn quickinfo_expression_type_not_changed_via_deletion() {
    let content = r#"type TypeEq<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;

const /*2*/test1: TypeEq<number[], [number, ...number[]]> = false;

declare const foo: [number, ...number[]];
declare const bar: number[];

const /*1*/test2: TypeEq<typeof foo, typeof bar> = false;"#;
    let mut s = Session::new_for_test("quickinfoExpressionTypeNotChangedViaDeletion", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyQuickInfoIs(t, "const test2: false", "")
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyQuickInfoIs(t, "const test1: false", "")
}
