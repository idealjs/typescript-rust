use tsox_lsp::fourslash::{self, Session};


#[test]
#[ignore = "需要 @link symlink 与 BaselineAutoImportsCompletions 基础设施（未移植）"]
fn auto_import_transitive_leak() {
    let mut s = Session::new_for_test("autoImportTransitiveLeak", "");
    fourslash::go_to_marker(&mut s, "fooCompletion");
    // TODO: f.VerifyCompletions(t, "fooCompletion", &fourslash.CompletionsExpectedList{
}
