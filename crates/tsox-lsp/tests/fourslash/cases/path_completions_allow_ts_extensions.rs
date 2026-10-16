use tsox_lsp::fourslash::{self, Session};


#[test]
fn path_completions_allow_ts_extensions() {
    let content = r#"// @moduleResolution: bundler
// @allowImportingTsExtensions: true
// @noEmit: true
// @Filename: /project/foo.ts
export const foo = 0;
// @Filename: /project/main.ts
import {} from ".//**/""#;
    let mut s = Session::new_for_test("pathCompletionsAllowTsExtensions", content);
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["foo"]);
    fourslash::set_user_preference(&mut s, "importModuleSpecifierEnding", "js");
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["foo.ts"]);
    fourslash::insert(&mut s, "foo.ts\"\nimport {} from \"./");
    fourslash::verify_completions_exact_at(&mut s, Some(""), &["foo.ts"]);
}
