use tsox_lsp::fourslash::Session;


#[test]
fn jsdoc_typedef_tag_go_to_definition() {
    let content = r#"// @lib: es5
// @allowNonTsExtensions: true
// @Filename: jsdocCompletion_typedef.js
/**
 * @typedef {Object} Person
 * @property {string} /*1*/personName
 * @property {number} personAge
 */

/**
 * @typedef {{ /*2*/animalName: string, animalAge: number }} Animal
 */

/** @type {Person} */
var person; person.[|personName/*3*/|]

/** @type {Animal} */
var animal; animal.[|animalName/*4*/|]"#;
    let _s = Session::new_for_test("jsdocTypedefTagGoToDefinition", content);
    // TODO: f.MarkTestAsStradaServer()
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "3", "4")
}
