use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: // Add a new export to the symlinked source package."]
#[test]
fn auto_import_symlinked_monorepo_source_update() {
    let mut s = Session::new("");
    // TODO: // Force auto import to build the cache (no exports yet).
    fourslash::go_to_marker(&mut s, "fooCompletion");
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{"fooCompletion"})
    // TODO: // Add a new export to the symlinked source package.
    fourslash::go_to_marker(&mut s, "fooEdit");
    fourslash::insert(&mut s, "\nexport function foo() {}");
    // TODO: // The new export should appear via granular cache update.
    fourslash::go_to_marker(&mut s, "fooCompletion");
    fourslash::unsupported("BaselineAutoImportsCompletions"); // f.BaselineAutoImportsCompletions(t, []string{"fooCompletion"})
}
