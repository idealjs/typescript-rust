use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_import_filtered_by_package_json_types_implicit() {
    let content = r#"//@noEmit: true
//@Filename: /package.json
{
  "dependencies": {
    "react": "*"
  }
}
//@Filename: /node_modules/@types/react/index.d.ts
export declare var React: any;
//@Filename: /node_modules/@types/react/package.json
{
  "name": "@types/react"
}
//@Filename: /node_modules/@types/fake-react/index.d.ts
export declare var ReactFake: any;
//@Filename: /node_modules/@types/fake-react/package.json
{
  "name": "@types/fake-react"
}
//@Filename: /src/index.ts
const x = Re/**/"#;
    let mut s = Session::new_for_test("completionsImport_filteredByPackageJson_typesImplicit", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
