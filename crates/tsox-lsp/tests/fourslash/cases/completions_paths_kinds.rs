use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"0", "1"}, &fourslash.CompletionsExpectedList{
}
