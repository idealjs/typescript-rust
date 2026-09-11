use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_in_augmented_class_module() {
    let content = r#"declare class m3f { foo(x: number): void }
namespace m3f { export interface I { foo(): void } }
var x: m3f./**/"#;
    let mut s = Session::new_for_test("completionInAugmentedClassModule", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["I"]);
}
