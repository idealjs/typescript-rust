use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_in_jsdoc_in_ts_file1() {
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
    let mut s = Session::new_for_test("quickInfoInJsdocInTsFile1", content);
    fourslash::verify_quick_info_at(&mut s, "1", "", "");
    fourslash::verify_quick_info_at(&mut s, "2", "", "");
    fourslash::verify_quick_info_at(&mut s, "3", "", "");
    fourslash::verify_quick_info_at(&mut s, "4", "type SomeObj = {\n    bar: string;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "5", "(parameter) stuff: {\n    quantity: number;\n}", "Stuff to do stuff with");
    fourslash::verify_quick_info_at(&mut s, "6", "(parameter) stuff: {\n    quantity: number;\n}", "Stuff to do stuff with");
    fourslash::verify_quick_info_at(&mut s, "7", "function doStuffWithStuff(stuff: {\n    quantity: number;\n}): void", "");
    fourslash::verify_quick_info_at(&mut s, "8", "", "");
}
