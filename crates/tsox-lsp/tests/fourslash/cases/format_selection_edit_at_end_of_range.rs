use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_selection_edit_at_end_of_range() {
    let content = r#"/*1*/var x = 1;/*2*/
void 0;"#;
    let mut s = Session::new_for_test("formatSelectionEditAtEndOfRange", content);
    // TODO: opts110 := f.GetOptions()
    // TODO: opts110.FormatCodeSettings.Semicolons = "remove"
    // TODO: f.Configure(t, opts110)
    fourslash::format_selection(&mut s, "1", "2");
    fourslash::verify_current_file_content(&mut s, r#"var x = 1
void 0;"#);
}
