use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_builder_locations_parameters() {
    let content = r#"var aa = 1;
class bar1{ constructor(/*1*/
class bar2{ constructor(a/*2*/
class bar3{ constructor(a, /*3*/
class bar4{ constructor(a, b/*4*/
class bar6{ constructor(public a, /*5*/
class bar7{ constructor(private a, /*6*/"#;
    let mut s = Session::new_for_test("completionListBuilderLocations_parameters", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
    // TODO: }
}
