use tsox_lsp::fourslash::{self, Session};


#[test]
fn jsdoc_callback_tag() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @lib: es5
// @strict: false
// @allowNonTsExtensions: true
// @Filename: jsdocCallbackTag.js
/**
 * @callback FooHandler - A kind of magic
 * @param {string} eventName - So many words
 * @param eventName2 {number | string} - Silence is golden
 * @param eventName3 - Osterreich mos def
 * @return {number} - DIVEKICK
 */
/**
 * @type {FooHa/*8*/ndler} callback
 */
var t/*1*/;

/**
 * @callback FooHandler2 - What, another one?
 * @param {string=} eventName - it keeps happening
 * @param {string} [eventName2] - i WARNED you dog
 */
/**
 * @type {FooH/*3*/andler2} callback
 */
var t2/*2*/;
t(/*4*/"!", /*5*/12, /*6*/false);"#;
    let mut s = Session::new_for_test("jsdocCallbackTag", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyQuickInfoIs(t, "var t: FooHandler", "")
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyQuickInfoIs(t, "var t2: FooHandler2", "")
    fourslash::go_to_marker(&mut s, "3");
    // TODO: f.VerifyQuickInfoIs(t, "type FooHandler2 = (eventName?: string | undefined, eventName2?: string) => 
    fourslash::go_to_marker(&mut s, "8");
    // TODO: f.VerifyQuickInfoIs(t, "type FooHandler = (eventName: string, eventName2: number | string, eventName
}
