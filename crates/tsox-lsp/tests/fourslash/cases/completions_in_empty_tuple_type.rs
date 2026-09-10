use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: // After `typeof` in a tuple type we are back in a value loc"]
#[test]
fn completions_in_empty_tuple_type() {
    let content = r#"type UserTuple = [["name", string], ["age", number], ["address", string]];
type AdminTuple = [/*1*/];
type OtherTuple = [string, /*2*/];
type QueryTuple = [typeof /*3*/];

const User: UserTuple = [["name", "2333"], ["age", 2333], ["address", "2333"]];"#;
    let mut s = Session::new_for_test("completionsInEmptyTupleType", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
    // TODO: // After `typeof` in a tuple type we are back in a value location, so type-only symbols shouldn't be
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
