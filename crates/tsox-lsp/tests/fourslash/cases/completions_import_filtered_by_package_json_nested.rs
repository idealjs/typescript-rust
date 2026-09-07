use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_import_filtered_by_package_json_nested() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"//@noEmit: true
//@Filename: /package.json
{
  "dependencies": {
    "react": "*"
  }
}
//@Filename: /node_modules/react/index.d.ts
export declare var React: any;
//@Filename: /node_modules/react/package.json
{
  "name": "react",
  "types": "./index.d.ts"
}
//@Filename: /dir/package.json
{
  "dependencies": {
    "redux": "*"
  }
}
//@Filename: /dir/node_modules/redux/package.json
{
  "name": "redux",
  "types": "./index.d.ts"
}
//@Filename: /dir/node_modules/redux/index.d.ts
export declare var Redux: any;
//@Filename: /dir/index.ts
const x = Re/**/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
