use tsox_lsp::fourslash::Session;


#[test]
fn no_completions_for_current_or_later_parameters_in_defaults() {
    let content = r#"function f1(a = /*1*/, b) { }
function f2(a = a/*2*/, b) { }
function f3(a = a + /*3*/, b = a/*4*/, c = /*5*/) { }
function f3(a) {
    function f4(b = /*6*/, c) { }
}
const f5 = (a, b = (c = /*7*/, e) => { }, d = b) => { }

type A1<K = /*T1*/, L> = K"#;
    let _s = Session::new_for_test("noCompletionsForCurrentOrLaterParametersInDefaults", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"3"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"4"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"5"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"6"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"7"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"T1"}, &fourslash.CompletionsExpectedList{
}
