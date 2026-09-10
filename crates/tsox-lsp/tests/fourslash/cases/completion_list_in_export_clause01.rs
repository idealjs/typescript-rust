use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn completion_list_in_export_clause01() {
    let content = r#"// @Filename: m1.ts
export var foo: number = 1;
export function bar() { return 10; }
export function baz() { return 10; }
// @Filename: m2.ts
export {/*1*/, /*2*/ from "./m1"
export {/*3*/} from "./m1"
export {foo,/*4*/ from "./m1"
export {bar as /*5*/, /*6*/ from "./m1"
export {foo, bar, baz as b,/*7*/} from "./m1""#;
    let mut s = Session::new_for_test("completionListInExportClause01", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2", "3"}, &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_empty_at(&mut s, Some("5"));
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_empty_at(&mut s, Some("7"));
    // TODO: }
}
