use tsox_lsp::fourslash::{self, Session};


#[test]
fn get_java_script_completions20() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @lib: es5
// @allowNonTsExtensions: true
// @Filename: file.js
/**
 * A person
 * @constructor
 * @param {string} name - The name of the person.
 * @param {number} age - The age of the person.
 */
function Person(name, age) {
    this.name = name;
    this.age = age;
}


Person.getName = 10;
Person.getNa/**/ = 10;"#;
    let mut s = Session::new_for_test("getJavaScriptCompletions20", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
