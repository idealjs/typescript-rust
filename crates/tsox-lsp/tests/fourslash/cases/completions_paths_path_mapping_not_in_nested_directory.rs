use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_paths_path_mapping_not_in_nested_directory() {
    let content = r#"// @Filename: /user.ts
import {} from "something//**/";
// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "baseUrl": ".",
        "paths": {
            "mapping/*": ["whatever"],
        }
    }
}"#;
    let mut s = Session::new_for_test("completionsPaths_pathMapping_notInNestedDirectory", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
