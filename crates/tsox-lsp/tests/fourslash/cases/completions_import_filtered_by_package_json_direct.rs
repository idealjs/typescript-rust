use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_import_filtered_by_package_json_direct() {
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
//@Filename: /node_modules/fake-react/index.d.ts
export declare var ReactFake: any;
//@Filename: /node_modules/fake-react/package.json
{
  "name": "fake-react",
  "types": "./index.d.ts"
}
//@Filename: /src/index.ts
const x = Re/**/"#;
    let mut s = Session::new_for_test("completionsImport_filteredByPackageJson_direct", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
