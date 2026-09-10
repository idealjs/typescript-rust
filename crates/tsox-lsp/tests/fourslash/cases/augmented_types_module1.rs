use tsox_lsp::fourslash::{self, Session};


#[test]
fn augmented_types_module1() {
    let content = r#"namespace m1c {
    export interface I { foo(): void; }
}
var m1c = 1; // Should be allowed
var x: m1c./*1*/;
var /*2*/r = m1c;"#;
    let mut s = Session::new_for_test("augmentedTypesModule1", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["I"]);
    fourslash::verify_quick_info_at(&mut s, "2", "var r: number", "");
}
