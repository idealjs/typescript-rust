use tsox_lsp::fourslash::{self, Session};


#[test]
#[ignore = "需要 @link symlink 与 BaselineAutoImportsCompletions 基础设施（未移植）"]
fn auto_import_symlinked_monorepo_source_update() {
    let mut s = Session::new_for_test("autoImportSymlinkedMonorepoSourceUpdate", "");
    // TODO: // Force auto import to build the cache (no exports yet).
    fourslash::go_to_marker(&mut s, "fooCompletion");
    // TODO: f.BaselineAutoImportsCompletions(t, []string{"fooCompletion"})
    // TODO: // Add a new export to the symlinked source package.
    fourslash::go_to_marker(&mut s, "fooEdit");
    fourslash::insert(&mut s, "\nexport function foo() {}");
    // TODO: // The new export should appear via granular cache update.
    fourslash::go_to_marker(&mut s, "fooCompletion");
    // TODO: f.BaselineAutoImportsCompletions(t, []string{"fooCompletion"})
}
