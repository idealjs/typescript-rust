use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_function_signatures8() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @allowJs: true
// @Filename: Foo.js
/**
 * Represents a person
 * a b multiline test
 * @constructor
 * @param {string} name The name of the person
 * @param {number} age The age of the person
 */
function Person(name, age) {
    this.name = name;
    this.age = age;
}
var p = new Pers/**/on();"#;
    let mut s = Session::new_for_test("jsDocFunctionSignatures8", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoIs(t, "constructor Person(name: string, age: number): Person", "Represents a person
}
