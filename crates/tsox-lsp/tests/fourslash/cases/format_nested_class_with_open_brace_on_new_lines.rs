use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: opts168 := f.GetOptions()"]
#[test]
fn format_nested_class_with_open_brace_on_new_lines() {
    let content = r#"module A
{
    class B {
        /*1*/
}"#;
    let mut s = Session::new_for_test("formatNestedClassWithOpenBraceOnNewLines", content);
    // TODO: opts168 := f.GetOptions()
    // TODO: opts168.FormatCodeSettings.PlaceOpenBraceOnNewLineForControlBlocks = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts168)
    // TODO: opts232 := f.GetOptions()
    // TODO: opts232.FormatCodeSettings.PlaceOpenBraceOnNewLineForFunctions = core.TSTrue
    fourslash::unsupported("Configure"); // f.Configure(t, opts232)
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("Insert"); // f.Insert(t, "}")
}
