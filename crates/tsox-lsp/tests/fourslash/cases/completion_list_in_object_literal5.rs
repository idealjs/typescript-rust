use tsox_lsp::fourslash::Session;


#[test]
fn completion_list_in_object_literal5() {
    let content = r#"const o = 'something' 
const obj = {
    prop: o/*1*/,
    pro() {
        const obj1 = {
            p:{
                s: {
                    h: {
                       hh: o/*2*/
                    },
                    someFun() {
                        o/*3*/
                    }
                }
            }
        }
    },
    o/*4*/
}"#;
    let _s = Session::new_for_test("completionListInObjectLiteral5", content);
    // TODO: f.VerifyCompletions(t, []string{"1"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"2"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"3"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"4"}, &fourslash.CompletionsExpectedList{
}
