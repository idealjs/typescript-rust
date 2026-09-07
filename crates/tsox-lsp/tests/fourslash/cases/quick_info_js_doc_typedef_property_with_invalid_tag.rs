use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "t1", "type MyType1 = {\n    name: string;\n    age: number;\n}", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "t2", "type MyType2 = {\n    name: string;\n    age: number;\n}", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "t3", "type MyType3 = {\n    name: string;\n    age: number;\n}", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "t4", "type MyType4 = {\n    name: string;\n    age: number;\n}", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "t5", "type MyType5 = {\n    name: string;\n}", ""+
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1n", "(property) name: string", "@-rule")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2n", "(property) name: string", "some comment")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3n", "(property) name: string", "@*stars")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4n", "(property) name: string", "@(parens)")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5n", "(property) name: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1a", "(property) age: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2a", "(property) age: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3a", "(property) age: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4a", "(property) age: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5a", "(property) age: number", "")
}
