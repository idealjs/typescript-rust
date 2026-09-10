use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_paths_path_mapping_parent_directory() {
    let content = r#"// @Filename: /src/a.ts
import { } from "foo//**/";
// @Filename: /oof/x.ts
export const x = 0;
// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "baseUrl": "src",
        "paths": {
            "foo/*": ["../oof/*"]
        }
    }
}"#;
    let mut s = Session::new_for_test("completionsPaths_pathMapping_parentDirectory", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
