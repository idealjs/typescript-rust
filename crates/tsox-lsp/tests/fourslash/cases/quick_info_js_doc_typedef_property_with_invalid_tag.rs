use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_js_doc_typedef_property_with_invalid_tag() {
    let content = r#"// @allowJs: true
// @Filename: /a.js
/**
 * @typedef {Object} MyType1
 * @property {string} name
 * @-rule
 * @property {number} age
 */

/**
 * @typedef {Object} MyType2
 * @property {string} name
 * some comment
 * @property {number} age
 */

/**
 * @typedef {Object} MyType3
 * @property {string} name
 * @*stars
 * @property {number} age
 */

/**
 * @typedef {Object} MyType4
 * @property {string} name
 * @(parens)
 * @property {number} age
 */

/**
 * @typedef {Object} MyType5
 * @property {string} name
 * @foo*bar
 * @property {number} age
 */

/** @type {/*t1*/MyType1} */
const obj1 = { /*1n*/name: "", /*1a*/age: 10 };

/** @type {/*t2*/MyType2} */
const obj2 = { /*2n*/name: "", /*2a*/age: 10 };

/** @type {/*t3*/MyType3} */
const obj3 = { /*3n*/name: "", /*3a*/age: 10 };

/** @type {/*t4*/MyType4} */
const obj4 = { /*4n*/name: "", /*4a*/age: 10 };

/** @type {/*t5*/MyType5} */
const obj5 = { /*5n*/name: "", /*5a*/age: 10 };
"#;
    let mut s = Session::new_for_test("quickInfoJSDocTypedefPropertyWithInvalidTag", content);
    fourslash::verify_quick_info_at(&mut s, "t1", "type MyType1 = {\n    name: string;\n    age: number;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "t2", "type MyType2 = {\n    name: string;\n    age: number;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "t3", "type MyType3 = {\n    name: string;\n    age: number;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "t4", "type MyType4 = {\n    name: string;\n    age: number;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "t5", "type MyType5 = {\n    name: string;\n}", concat!("", "\n\n*@foo* — *bar", "\n\n*@property* — {number} age"));
    fourslash::verify_quick_info_at(&mut s, "1n", "(property) name: string", "@-rule");
    fourslash::verify_quick_info_at(&mut s, "2n", "(property) name: string", "some comment");
    fourslash::verify_quick_info_at(&mut s, "3n", "(property) name: string", "@*stars");
    fourslash::verify_quick_info_at(&mut s, "4n", "(property) name: string", "@(parens)");
    fourslash::verify_quick_info_at(&mut s, "5n", "(property) name: string", "");
    fourslash::verify_quick_info_at(&mut s, "1a", "(property) age: number", "");
    fourslash::verify_quick_info_at(&mut s, "2a", "(property) age: number", "");
    fourslash::verify_quick_info_at(&mut s, "3a", "(property) age: number", "");
    fourslash::verify_quick_info_at(&mut s, "4a", "(property) age: number", "");
    fourslash::verify_quick_info_at(&mut s, "5a", "(property) age: number", "");
}
