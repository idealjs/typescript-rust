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
    let mut s = Session::new_for_test("jsdocTypedefTag", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::verify_completions_include_exclude_at(&mut s, Some("numberLike"), &["charAt", "toExponential"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("person"), &["personName", "personAge"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("personName"), &["charAt"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("personAge"), &["toExponential"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("animal"), &["animalName", "animalAge"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("animalName"), &["charAt"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("animalAge"), &["toExponential"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("dog"), &["dogName", "dogAge"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("dogName"), &["charAt"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("dogAge"), &["toExponential"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("cat"), &["catName", "catAge"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("catName"), &["charAt"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("catAge"), &["toExponential"], &[]);
    fourslash::verify_quick_info_at(&mut s, "AnimalType", "type Animal = {\n    animalName: string;\n    animalAge: number;\n}", "- think Giraffes");
}
