use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_import_or_export_specifier() {
    let content = r#"// @Filename: exports.ts
export let foo = 1;
let someValue = 2;
let someType = 3;
type someType2 = 4;
export {
  someValue as "__some value",
  someType as "__some type",
  type someType2 as "__some type2",
};
// @Filename: values.ts
import { /*valueImport0*/ } from "./exports";
import { /*valueImport1*/ as valueImport1 } from "./exports";
import { foo as /*valueImport2*/ } from "./exports";
import { foo, /*valueImport3*/ as valueImport3 } from "./exports";
import * as _a from "./exports";
_a./*namespaceImport1*/;

export { /*valueExport0*/ } from "./exports";
export { /*valueExport1*/ as valueExport1 } from "./exports";
export { foo as /*valueExport2*/ } from "./exports";
export { foo, /*valueExport3*/ } from "./exports";
// @Filename: types.ts
import { type /*typeImport0*/ } from "./exports";
import { type /*typeImport1*/ as typeImport1 } from "./exports";
import { type foo as /*typeImport2*/ } from "./exports";
import { type foo, type /*typeImport3*/ as typeImport3 } from "./exports";
import * as _a from "./exports";

export { type /*typeExport0*/ } from "./exports";
export { type /*typeExport1*/ as typeExport1 } from "./exports";
export { type foo as /*typeExport2*/ } from "./exports";
export { type foo, type /*typeExport3*/ } from "./exports";"#;
    let mut s = Session::new_for_test("completionsImportOrExportSpecifier", content);
    fourslash::go_to_marker(&mut s, "valueImport0");
    // TODO: f.VerifyCompletions(t, "valueImport0", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "valueImport1");
    // TODO: f.VerifyCompletions(t, "valueImport1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "valueImport2");
    // TODO: f.VerifyCompletions(t, "valueImport2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "valueImport3");
    // TODO: f.VerifyCompletions(t, "valueImport3", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("namespaceImport1"), &["foo"]);
    fourslash::go_to_marker(&mut s, "valueExport0");
    // TODO: f.VerifyCompletions(t, "valueExport0", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "valueExport1");
    // TODO: f.VerifyCompletions(t, "valueExport1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "valueExport2");
    // TODO: f.VerifyCompletions(t, "valueExport2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "valueExport3");
    // TODO: f.VerifyCompletions(t, "valueExport3", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "typeImport0");
    // TODO: f.VerifyCompletions(t, "typeImport0", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "typeImport1");
    // TODO: f.VerifyCompletions(t, "typeImport1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "typeImport2");
    // TODO: f.VerifyCompletions(t, "typeImport2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "typeImport3");
    // TODO: f.VerifyCompletions(t, "typeImport3", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "typeExport0");
    // TODO: f.VerifyCompletions(t, "typeExport0", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "typeExport1");
    // TODO: f.VerifyCompletions(t, "typeExport1", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "typeExport2");
    // TODO: f.VerifyCompletions(t, "typeExport2", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "typeExport3");
    // TODO: f.VerifyCompletions(t, "typeExport3", &fourslash.CompletionsExpectedList{
}
