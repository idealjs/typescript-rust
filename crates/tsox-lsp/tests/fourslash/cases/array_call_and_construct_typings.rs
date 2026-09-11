use tsox_lsp::fourslash::{self, Session};


#[test]
fn array_call_and_construct_typings() {
    let content = r#"var a/*1*/1 = new Array();
var a/*2*/2 = new Array(1);
var a/*3*/3 = new Array<boolean>();
var a/*4*/4 = new Array<boolean>(1);
var a/*5*/5 = new Array("s");
var a/*6*/6 = Array();
var a/*7*/7 = Array(1);
var a/*8*/8 = Array<boolean>();
var a/*9*/9 = Array<boolean>(1);
var a/*10*/10 = Array("s");"#;
    let mut s = Session::new_for_test("arrayCallAndConstructTypings", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var a1: any[]", "");
    fourslash::verify_quick_info_at(&mut s, "2", "var a2: any[]", "");
    fourslash::verify_quick_info_at(&mut s, "3", "var a3: boolean[]", "");
    fourslash::verify_quick_info_at(&mut s, "4", "var a4: boolean[]", "");
    fourslash::verify_quick_info_at(&mut s, "5", "var a5: string[]", "");
    fourslash::verify_quick_info_at(&mut s, "6", "var a6: any[]", "");
    fourslash::verify_quick_info_at(&mut s, "7", "var a7: any[]", "");
    fourslash::verify_quick_info_at(&mut s, "8", "var a8: boolean[]", "");
    fourslash::verify_quick_info_at(&mut s, "9", "var a9: boolean[]", "");
    fourslash::verify_quick_info_at(&mut s, "10", "var a10: string[]", "");
}
