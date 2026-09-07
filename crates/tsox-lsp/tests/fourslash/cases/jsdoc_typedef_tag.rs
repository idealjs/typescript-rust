use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn jsdoc_typedef_tag() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @lib: es5
// @allowNonTsExtensions: true
// @Filename: jsdocCompletion_typedef.js
/** @typedef {(string | number)} NumberLike */

/**
 * @typedef Animal - think Giraffes
 * @type {Object}
 * @property {string} animalName
 * @property {number} animalAge
 */

/**
 * @typedef {Object} Person
 * @property {string} personName
 * @property {number} personAge
 */

/**
 * @typedef {Object}
 * @property {string} catName
 * @property {number} catAge
 */
var Cat;

/** @typedef {{ dogName: string, dogAge: number }} */
var Dog;

/** @type {NumberLike} */
var numberLike; numberLike./*numberLike*/

/** @type {Person} */
var p;p./*person*/;
p.personName./*personName*/;
p.personAge./*personAge*/;

/** @type {/*AnimalType*/Animal} */
var a;a./*animal*/;
a.animalName./*animalName*/;
a.animalAge./*animalAge*/;

/** @type {Cat} */
var c;c./*cat*/;
c.catName./*catName*/;
c.catAge./*catAge*/;

/** @type {Dog} */
var d;d./*dog*/;
d.dogName./*dogName*/;
d.dogAge./*dogAge*/;"#;
    let mut s = Session::new(content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "numberLike", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "person", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "personName", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "personAge", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "animal", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "animalName", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "animalAge", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "dog", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "dogName", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "dogAge", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "cat", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "catName", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "catAge", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "AnimalType", "type Animal = {\n    animalName: string;\n    animalAge: numbe
}
