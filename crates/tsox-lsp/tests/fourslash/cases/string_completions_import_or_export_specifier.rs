use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn string_completions_import_or_export_specifier() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @Filename: exports.ts
export let foo = 1;
let someValue = 2;
let someType = 3;
export {
  someValue as "__some value",
  someType as "__some type",
};
// @Filename: values.ts
import { "/*valueImport0*/" } from "./exports";
import { "/*valueImport1*/" as valueImport1 } from "./exports";
import { foo as "/*valueImport2*/" } from "./exports";
import { foo, "/*valueImport3*/" as valueImport3 } from "./exports";

export { "/*valueExport0*/" } from "./exports";
export { "/*valueExport1*/" as valueExport1 } from "./exports";
export { foo as "/*valueExport2*/" } from "./exports";
export { foo, "/*valueExport3*/" } from "./exports";
// @Filename: types.ts
import { type "/*typeImport0*/" } from "./exports";
import { type "/*typeImport1*/" as typeImport1 } from "./exports";
import { type foo as "/*typeImport2*/" } from "./exports";
import { type foo, type "/*typeImport3*/" as typeImport3 } from "./exports";

export { type "/*typeExport0*/" } from "./exports";
export { type "/*typeExport1*/" as typeExport1 } from "./exports";
export { type foo as "/*typeExport2*/" } from "./exports";
export { type foo, type "/*typeExport3*/" } from "./exports";"#;
    let mut s = Session::new_for_test("stringCompletionsImportOrExportSpecifier", content);
    fourslash::verify_completions_exact_at(&mut s, Some("valueImport0"), &["__some type", "__some value", "foo"]);
    fourslash::verify_completions_exact_at(&mut s, Some("valueImport1"), &["__some type", "__some value", "foo"]);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "valueImport2", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("valueImport3"), &["__some type", "__some value"]);
    fourslash::verify_completions_exact_at(&mut s, Some("valueExport0"), &["__some type", "__some value", "foo"]);
    fourslash::verify_completions_exact_at(&mut s, Some("valueExport1"), &["__some type", "__some value", "foo"]);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "valueExport2", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("valueExport3"), &["__some type", "__some value"]);
    fourslash::verify_completions_exact_at(&mut s, Some("typeImport0"), &["__some type", "__some value", "foo"]);
    fourslash::verify_completions_exact_at(&mut s, Some("typeImport1"), &["__some type", "__some value", "foo"]);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "typeImport2", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("typeImport3"), &["__some type", "__some value"]);
    fourslash::verify_completions_exact_at(&mut s, Some("typeExport0"), &["__some type", "__some value", "foo"]);
    fourslash::verify_completions_exact_at(&mut s, Some("typeExport1"), &["__some type", "__some value", "foo"]);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "typeExport2", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("typeExport3"), &["__some type", "__some value"]);
}
