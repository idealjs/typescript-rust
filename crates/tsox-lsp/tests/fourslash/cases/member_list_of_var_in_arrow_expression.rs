use tsox_lsp::fourslash::{self, Session};


#[test]
fn member_list_of_var_in_arrow_expression() {
    let content = r#"interface IMap<T> {
    [key: string]: T;
}
var map: IMap<{ a1: string; }[]>;
var categories: string[];
each(categories, category => {
    var changes = map[category];
    changes[0]./*1*/a1;
    return each(changes, change => {
    });
});
function each<T>(items: T[], handler: (item: T) => void) { }"#;
    let mut s = Session::new_for_test("memberListOfVarInArrowExpression", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(property) a1: string", "");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
