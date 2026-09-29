use tsox_lsp::fourslash::{self, Session};


#[test]
fn augmented_types_module5() {
    let content = r#"declare class m3e { foo(): void }
namespace m3e { export var y = 2; }
var /*1*/r = new m3e();
r./*2*/
var /*4*/r2 = m3e./*3*/"#;
    let mut s = Session::new_for_test("augmentedTypesModule5", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var r: m3e", "");
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["foo"]);
    fourslash::insert(&mut s, "foo();");
    fourslash::verify_completions_include_exclude_at(&mut s, Some("3"), &["y"], &[]);
    fourslash::insert(&mut s, "y;");
    fourslash::verify_quick_info_at(&mut s, "4", "var r2: number", "");
}
