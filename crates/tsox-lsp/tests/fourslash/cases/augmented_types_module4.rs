use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn augmented_types_module4() {
    let content = r#"namespace m3d { export var y = 2; }
declare class m3d { foo(): void }
var /*1*/r = new m3d();
r./*2*/
var /*4*/r2 = m3d./*3*/"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "var r: m3d", "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "foo();");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "y;");
    fourslash::verify_quick_info_at(&mut s, "4", "var r2: number", "");
}
