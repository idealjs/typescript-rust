use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: completions := f.GetCompletions(t, nil /*userPreferences*/)"]
#[test]
fn completion_resolve_after_edit() {
    let content = r#"
// @filename: a.ts
interface I {
	x: number;
	y: number;
}
declare const u: I;
/*a*/

// @filename: 1.ts
/*b*/
"#;
    let mut s = Session::new_for_test("completionResolveAfterEdit", content);
    fourslash::go_to_marker(&mut s, "a");
    // TODO: completions := f.GetCompletions(t, nil /*userPreferences*/)
    // TODO: if completions == nil || len(completions.Items) == 0 {
    // TODO: firstItem := completions.Items[0]
    fourslash::go_to_marker(&mut s, "b");
    fourslash::insert(&mut s, "1");
    // TODO: resolved := f.ResolveCompletionItem(t, firstItem)
    // TODO: if resolved == nil {
}

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn resolve_import_statement_completion() {
    let content = r#"
// @filename: a.ts
export const u = 1;

// @filename: 1.ts
[|import u/*a*/|]
"#;
    let mut s = Session::new_for_test("resolveImportStatementCompletion", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
}
