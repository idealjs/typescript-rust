use tsox_lsp::fourslash::{self, Session};


#[test]
fn generic_combinator_with_constraints1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function apply<T, U extends Date>(source: T[], selector: (x: T) => U) {
    var /*1*/xs = source.map(selector); // any[]
    var /*2*/xs2 = source.map((x: T, a, b): U => { return null }); // any[] 
}"#;
    let mut s = Session::new_for_test("genericCombinatorWithConstraints1", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(local var) xs: U[]", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(local var) xs2: U[]", "");
}
