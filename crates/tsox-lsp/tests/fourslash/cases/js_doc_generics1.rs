use tsox_lsp::fourslash::Session;


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
    let _s = Session::new_for_test("jsDocGenerics1", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
    // TODO: }
}
