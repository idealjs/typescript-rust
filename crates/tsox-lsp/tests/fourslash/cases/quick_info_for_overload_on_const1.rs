use tsox_lsp::fourslash::{self, Session};

#[test]
fn quick_info_for_overload_on_const1() {
    let content = r#"interface I {
    x/*1*/1(a: number, callback: (x: 'hi') => number);
}
class C {
    x/*2*/1(a: number, call/*3*/back: (x: 'hi') => number);
    x/*4*/1(a: number, call/*5*/back: (x: string) => number) {
        call/*6*/back('hi');
        callback('bye');
        var hm = "hm";
        callback(hm);
    }
}
var c: C;
c.x/*7*/1(1, (x/*8*/x: 'hi') => { return 1; } );
c.x1(1, (x/*9*/x: 'bye') => { return 1; } );
c.x1(1, (x/*10*/x) => { return 1; } );"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "1",
        "(method) I.x1(a: number, callback: (x: 'hi') => number): any",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "2",
        "(method) C.x1(a: number, callback: (x: 'hi') => number): any",
        "",
    );
    fourslash::verify_quick_info_at(&mut s, "3", "(parameter) callback: (x: 'hi') => number", "");
    fourslash::verify_quick_info_at(
        &mut s,
        "4",
        "(method) C.x1(a: number, callback: (x: string) => number): void",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "5",
        "(parameter) callback: (x: string) => number",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "6",
        "(parameter) callback: (x: string) => number",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "7",
        "(method) C.x1(a: number, callback: (x: 'hi') => number): any",
        "",
    );
    fourslash::verify_quick_info_at(&mut s, "8", "(parameter) xx: \"hi\"", "");
    fourslash::verify_quick_info_at(&mut s, "9", "(parameter) xx: \"bye\"", "");
    fourslash::verify_quick_info_at(&mut s, "10", "(parameter) xx: \"hi\"", "");
}
