use tsox_lsp::fourslash::{self, Session};


#[test]
fn js_doc_type_tag_quick_info1() {
    let content = r#"// @lib: es5
// @strict: true
// @allowJs: true
// @Filename: jsDocTypeTag1.js
/** @type {String} */
var /*1*/S;
/** @type {Number} */
var /*2*/N;
/** @type {Boolean} */
var /*3*/B;
/** @type {Void} */
var /*4*/V;
/** @type {Undefined} */
var /*5*/U;
/** @type {Null} */
var /*6*/Nl;
/** @type {Array} */
var /*7*/A;
/** @type {Promise} */
var /*8*/P;
/** @type {Object} */
var /*9*/Obj;
/** @type {Function} */
var /*10*/Func;
/** @type {*} */
var /*11*/AnyType;
/** @type {?} */
var /*12*/QType;
/** @type {String|Number} */
var /*13*/SOrN;"#;
    let mut s = Session::new_for_test("jsDocTypeTagQuickInfo1", content);
    // TODO: f.VerifyBaselineHover(t)
}
