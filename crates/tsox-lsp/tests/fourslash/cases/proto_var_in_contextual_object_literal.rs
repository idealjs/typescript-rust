use tsox_lsp::fourslash::{self, Session};


#[test]
fn proto_var_in_contextual_object_literal() {
    let content = r#"var o1 : {
    __proto__: number;
    p: number;
} = {
        /*1*/
    };
var o2: {
    __proto__: number;
    p: number;
} = {
        /*2*/
    };
var o3: {
    "__proto__": number;
    p: number;
} = {
        /*3*/
    };
var o4: {
    "__proto__": number;
    p: number;
} = {
        /*4*/
    };
var o5: {
    __proto__: number;
    ___proto__: string;
    p: number;
} = {
        /*5*/
    };
var o6: {
    __proto__: number;
    ___proto__: string;
    p: number;
} = {
        /*6*/
    };"#;
    let mut s = Session::new_for_test("protoVarInContextualObjectLiteral", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "__proto__: 10,");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "\"__proto__\": 10,");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "__proto__: 10,");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "4");
    // TODO: f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "\"__proto__\": 10,");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "5");
    // TODO: f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "__proto__: 10,");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "\"___proto__\": \"10\",");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "6");
    // TODO: f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "___proto__: \"10\",");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "\"__proto__\": 10,");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
