use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn indirect_class_instantiation() {
    let content = r#"// @allowJs: true
// @Filename: something.js
function TestObj(){
    this.property = "value";
}
var constructor = TestObj;
var instance = new constructor();
instance./*a*/
var class2 = function() { };
class2.prototype.blah = function() { };
var inst2 = new class2();
inst2.blah/*b*/;"#;
    let mut s = Session::new_for_test("indirectClassInstantiation", content);
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
    // TODO: f.Backspace(t, 1)
    fourslash::go_to_marker(&mut s, "b");
    // TODO: f.VerifyQuickInfoIs(t, "(method) class2.blah(): void", "")
}
