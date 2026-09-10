use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: }"]
#[test]
fn js_doc_generics1() {
    let content = r#"// @allowNonTsExtensions: true
// @Filename: ref.d.ts
namespace Thing {
    export interface Thung {
        a: number;
    ]
]
// @Filename: Foo.js

/** @type {Array<number>} */
var v;
v[0]./*1*/

/** @type {{x: Array<Array<number>>}} */
var w;
w.x[0][0]./*2*/

/** @type {Array<Thing.Thung>} */
var x;
x[0].a./*3*/"#;
    let mut s = Session::new_for_test("jsDocGenerics1", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
    // TODO: }
}
