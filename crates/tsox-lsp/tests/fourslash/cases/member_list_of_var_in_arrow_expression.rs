use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) a1: string", "")
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
