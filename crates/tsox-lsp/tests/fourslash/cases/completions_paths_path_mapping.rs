use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_paths_path_mapping() {
    let content = r#"// @Filename: /src/b.ts
export const x = 0;
// @Filename: /src/dir/x.ts
/export const x = 0;
// @Filename: /src/a.ts
import {} from "foo//*0*/";
import {} from "foo/dir//*1*/";
// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "baseUrl": ".",
        "paths": {
            "foo/*": ["src/*"]
        }
    }
}"#;
    let mut s = Session::new_for_test("completionsPaths_pathMapping", content);
    // TODO: f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
