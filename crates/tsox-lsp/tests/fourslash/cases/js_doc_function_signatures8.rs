use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
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
    let mut s = Session::new(content);
    fourslash::go_to_marker(&mut s, "");
    fourslash::unsupported("VerifyQuickInfoIs"); // f.VerifyQuickInfoIs(t, "constructor Person(name: string, age: number): Person", "Represents a person
}
