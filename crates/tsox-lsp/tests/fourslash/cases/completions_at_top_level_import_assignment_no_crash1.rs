use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_at_top_level_import_assignment_no_crash1() {
    let content = r#"// @filename: /a.ts
import x =/*1*/
class Foo {}
// @filename: /b.ts
import x = /*2*/
class Foo {}
// @filename: /c.ts
import x =/*3*/
"#;
    let mut s = Session::new_for_test("completionsAtTopLevelImportAssignmentNoCrash1", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2", "3"}, &fourslash.CompletionsExpectedList{
}
