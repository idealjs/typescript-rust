use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: 变量 TestAutoImportTransitiveLeakScenario 非标准 content"]
#[test]
fn auto_import_transitive_leak() {
    let mut s = Session::new_for_test("autoImportTransitiveLeak", "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "fooCompletion", &fourslash.CompletionsExpectedList{
}
