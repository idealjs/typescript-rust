use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_var_in_arrow_expression() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"interface IMap<T> {
    [key: string]: T;
}
var map: IMap<string[]>;
var categories: string[];
each(categories, category => {
    var /*1*/changes = map[category];
    return each(changes, change => {
    });
});
function each<T>(items: T[], handler: (item: T) => void) { }"#;
    let mut s = Session::new_for_test("quickInfoOnVarInArrowExpression", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(local var) changes: string[]", "");
}
