use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_entry_for_property_from_union_of_module_type() {
    let content = r#"namespace E {
    export var n = 1;
    export var x = 0;
}
namespace F {
    export var n = 1;
    export var y = 0;
}
var q: typeof E | typeof F;
var j = q./*1*/"#;
    let mut s = Session::new_for_test("completionEntryForPropertyFromUnionOfModuleType", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
