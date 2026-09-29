use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_after_invalid_character() {
    let content = r#"// Completion after invalid character
namespace testModule {
    export var foo = 1;
}
@
testModule./**/"#;
    let mut s = Session::new_for_test("completionListAfterInvalidCharacter", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["foo"]);
}
