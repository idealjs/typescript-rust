use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_in_jsdoc_in_ts_file1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"/** @type {() => { /*1*/data: string[] }} */
function test(): { data: string[] } {
  return {
    data: [],
  };
}

/** @returns {{ /*2*/data: string[] }} */
function test2(): { data: string[] } {
  return {
    data: [],
  };
}

/** @type {{ /*3*/bar: string; }} */
const test3 = { bar: '' };

type SomeObj = { bar: string; };
/** @type {SomeObj/*4*/} */
const test4 = { bar: '' }

/**
 * @param/*5*/ stuff/*6*/ Stuff to do stuff with
 */
function doStuffWithStuff(stuff: { quantity: number }) {}

declare const stuff: { quantity: number };
/** @see {doStuffWithStuff/*7*/} */
if (stuff.quantity) {}

/** @type {(a/*8*/: string) => void} */
function test2(a: string) {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "type SomeObj = {\n    bar: string;\n}", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "(parameter) stuff: {\n    quantity: number;\n}", "Stuff to do stuff wit
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6", "(parameter) stuff: {\n    quantity: number;\n}", "Stuff to do stuff wit
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "7", "function doStuffWithStuff(stuff: {\n    quantity: number;\n}): void", "
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "8", "", "")
}
