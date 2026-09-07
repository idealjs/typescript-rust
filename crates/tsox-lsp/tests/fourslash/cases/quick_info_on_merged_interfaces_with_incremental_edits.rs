use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.Backspace"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "1");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "(property) B<string>.bar: string", "")
    fourslash::unsupported("DeleteAtCaret"); // f.DeleteAtCaret(t, 1)
    fourslash::insert(&mut s, "z");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "any", "")
    fourslash::unsupported("VerifyNumberOfErrorsInCurrentFile"); // f.VerifyNumberOfErrorsInCurrentFile(t, 1)
    fourslash::unsupported("Backspace"); // f.Backspace(t, 1)
    fourslash::insert(&mut s, "a");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "(property) B<string>.bar: string", "")
    fourslash::go_to_marker(&mut s, "2");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "var r4: string", "")
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
