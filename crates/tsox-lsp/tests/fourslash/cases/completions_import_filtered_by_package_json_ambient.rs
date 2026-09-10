use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_import_filtered_by_package_json_ambient() {
    let content = r#"// @lib: es5
//@noEmit: true
//@Filename: /package.json
{
  "dependencies": {
    "react-syntax-highlighter": "*",
    "declared-by-foo": "*"
  }
}
//@Filename: /node_modules/@types/foo/index.d.ts
declare module "foo" {
  export const foo: any;
}
declare module "declared-by-foo" {
  export const declaredBySomethingNotInPackageJson: any;
}
//@Filename: /node_modules/@types/foo/package.json
{
  "name": "@types/node"
}
//@Filename: /node_modules/@types/react-syntax-highlighter/index.d.ts
declare module "react-syntax-highlighter/sub" {
  const agate: any;
  export default agate;
}
declare module "something-else" {
  export const somethingElse: any;  
}
//@Filename: /node_modules/@types/react-syntax-highlighter/package.json
{
  "name": "@types/react-syntax-highlighter"
}
//@Filename: /src/ambient.ts
declare module "local" {
  export const local: any';
}
//@Filename: /src/index.ts
fo/*1*/
aga/*2*/
somethi/*3*/
declaredBy/*4*/
loca/*5*/"#;
    let mut s = Session::new_for_test("completionsImport_filteredByPackageJson_ambient", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
}
