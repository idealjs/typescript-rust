use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn auto_imports_with_root_dirs_and_rooted_path01() {
    let content = r#"// @Filename: /dir/foo.ts
 export function foo() {}
// @Filename: /dir/bar.ts
 /*$*/
// @Filename: /dir/tsconfig.json
{
    "compilerOptions": {
        "module": "commonjs",
        "moduleResolution": "classic",
        "rootDirs": ["D:/"]
    }
}"#;
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "$");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
