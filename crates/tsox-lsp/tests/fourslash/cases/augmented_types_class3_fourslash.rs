use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn augmented_types_class3_fourslash() {
    let content = r#"class c/*1*/5b { public foo() { } }
namespace c/*2*/5b { export var y = 2; } // should be ok
/*3*/"#;
    let mut s = Session::new_for_test("augmentedTypesClass3Fourslash", content);
    fourslash::verify_quick_info_at(&mut s, "1", "class c5b\nnamespace c5b", "");
    fourslash::verify_quick_info_at(&mut s, "2", "class c5b\nnamespace c5b", "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
