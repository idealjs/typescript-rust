use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_paths_path_mapping_non_trailing_wildcard1() {
    let content = r#"// @Filename: /src/b.ts
export const x = 0;
// @Filename: /src/dir/x.ts
/export const x = 0;
// @Filename: /src/a.ts
import {} from "foo//*0*/";
import {} from "foo/dir//*1*/"; // invalid
import {} from "foo/[|_|]/*2*/";
import {} from "foo/_dir//*3*/";
// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "baseUrl": ".",
        "paths": {
            "foo/_*/suffix": ["src/*.ts"]
        }
    }
}"#;
    let mut s = Session::new_for_test("completionsPaths_pathMapping_nonTrailingWildcard1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_empty_at(&mut s, Some("1"));
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
