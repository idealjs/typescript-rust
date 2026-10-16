use tsox_lsp::fourslash::Session;


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completion_list_without_variableinitializer() {
    let content = r#"const a = a/*1*/;
const b = a && b/*2*/;
const c = [{ prop: [c/*3*/] }];
const d = () => { d/*4*/ };
const e = () => expression/*5*/
const f = { prop() { e/*6*/ }  };
const fn = (p = /*7*/) => {}
const { g, h = /*8*/ } = { ... }
const [ g1, h1 = /*9*/ ] = [ ... ]
const { a1 } = a/*10*/;
const { a2 } = fn({a: a/*11*/});
const [ a3 ] = a/*12*/;
const [ a4 ] = fn([a/*13*/]);"#;
    let _s = Session::new_for_test("completionListWithoutVariableinitializer", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"2"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"3"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"4"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"5"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"6"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"7"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"8"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"9"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"10"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"11"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"12"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"13"}, &fourslash.CompletionsExpectedList{
}
