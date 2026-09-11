use tsox_lsp::fourslash::{self, Session};


#[test]
fn special_intersections_order_independent() {
    let content = r#"declare function a(arg: 'test' | (string & {})): void
a('/*1*/')
declare function b(arg: 'test' | ({} & string)): void
b('/*2*/')"#;
    let mut s = Session::new_for_test("specialIntersectionsOrderIndependent", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}
