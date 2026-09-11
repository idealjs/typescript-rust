use tsox_lsp::fourslash::{self, Session};


#[test]
fn java_script_class4() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: Foo.js
class Foo {
   constructor() {
       /**
         * @type {string}
       */
       this.baz = null;
   }
}
var x = new Foo();
x/**/"#;
    let mut s = Session::new_for_test("javaScriptClass4", content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::insert(&mut s, ".baz.");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
