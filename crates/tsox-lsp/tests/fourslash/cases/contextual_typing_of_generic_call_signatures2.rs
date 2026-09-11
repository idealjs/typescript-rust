use tsox_lsp::fourslash::{self, Session};


#[test]
fn contextual_typing_of_generic_call_signatures2() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface I {
    <T>(x: T): void
}
function f6(x: <T extends I>(p: T) => void) { }
// x should not be contextually typed so this should be an error
f6(/**/x => x<number>())"#;
    let mut s = Session::new_for_test("contextualTypingOfGenericCallSignatures2", content);
    fourslash::verify_quick_info_at(&mut s, "", "(parameter) x: T extends I", "");
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
}
