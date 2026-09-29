use tsox_lsp::fourslash::{self, Session};


#[test]
fn augmented_types_module4() {
    let content = r#"namespace m3d { export var y = 2; }
declare class m3d { foo(): void }
var /*1*/r = new m3d();
r./*2*/
var /*4*/r2 = m3d./*3*/"#;
    let mut s = Session::new_for_test("augmentedTypesModule4", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var r: m3d", "");
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["foo"]);
    fourslash::insert(&mut s, "foo();");
    fourslash::verify_completions_include_exclude_at(&mut s, Some("3"), &["y"], &[]);
    fourslash::insert(&mut s, "y;");
    fourslash::verify_quick_info_at(&mut s, "4", "var r2: number", "");
}
