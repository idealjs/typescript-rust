use tsox_lsp::fourslash::{self, Session};


#[test]
fn auto_import_transitive_leak() {
    let mut s = Session::new_for_test("autoImportTransitiveLeak", "");
    fourslash::go_to_marker(&mut s, "fooCompletion");
    // TODO: f.VerifyCompletions(t, "fooCompletion", &fourslash.CompletionsExpectedList{
}
