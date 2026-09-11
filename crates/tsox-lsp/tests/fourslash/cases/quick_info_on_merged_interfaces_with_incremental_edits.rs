use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_on_merged_interfaces_with_incremental_edits() {
    let content = r#"// @strict: false
namespace MM {
    interface B<T> {
        foo: number;
    }
    interface B<T> {
        bar: string;
    }
    var b: B<string>;
    var r3 = b.foo; // number
    var r/*2*/4 = b.b/*1*/ar; // string
}"#;
    let mut s = Session::new_for_test("quickInfoOnMergedInterfacesWithIncrementalEdits", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyQuickInfoIs(t, "(property) B<string>.bar: string", "")
    // TODO: f.DeleteAtCaret(t, 1)
    fourslash::insert(&mut s, "z");
    // TODO: f.VerifyQuickInfoIs(t, "any", "")
    fourslash::verify_number_of_errors_in_current_file(&mut s, 1);
    // TODO: f.Backspace(t, 1)
    fourslash::insert(&mut s, "a");
    // TODO: f.VerifyQuickInfoIs(t, "(property) B<string>.bar: string", "")
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyQuickInfoIs(t, "var r4: string", "")
    fourslash::verify_no_errors(&mut s, );
}
