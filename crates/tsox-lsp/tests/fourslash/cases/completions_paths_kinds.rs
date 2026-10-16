use tsox_lsp::fourslash::Session;


#[test]
fn completions_paths_kinds() {
    let content = r#"// @Filename: /src/b.ts
not read
// @Filename: /src/dir/x.ts
not read
// @Filename: /src/a.ts
import {} from "./[|/*0*/|]";
import {} from "./[|/*1*/|]";
// @Filename: /tsconfig.json
{
    "compilerOptions": {
        "baseUrl": ".",
        "paths": {
            "foo/*": ["src/*"]
        }
    }
}"#;
    let _s = Session::new_for_test("completionsPaths_kinds", content);
    // TODO: f.VerifyCompletions(t, []string{"0", "1"}, &fourslash.CompletionsExpectedList{
}
