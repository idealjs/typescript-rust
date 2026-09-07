use tsox_lsp::fourslash::{self, Session};

#[test]
fn contextually_typed_parameters() {
    let content = r#"declare function foo(cb: (this: any, x: number, y: string, z: boolean) => void): void;

foo(function(this, a, ...args) {
    a/*10*/;
    args/*11*/;
});

foo(function(this, a, b, ...args) {
    a/*20*/;
    b/*21*/;
    args/*22*/;
});

foo(function(this, a, b, c, ...args) {
    a/*30*/;
    b/*31*/;
    c/*32*/;
    args/*33*/;
});

foo(function(a, ...args) {
    a/*40*/;
    args/*41*/;
});

foo(function(a, b, ...args) {
    a/*50*/;
    b/*51*/;
    args/*52*/;
});

foo(function(a, b, c, ...args) {
    a/*60*/;
    b/*61*/;
    c/*62*/;
    args/*63*/;
});"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "10", "(parameter) a: number", "");
    fourslash::verify_quick_info_at(
        &mut s,
        "11",
        "(parameter) args: [y: string, z: boolean]",
        "",
    );
    fourslash::verify_quick_info_at(&mut s, "20", "(parameter) a: number", "");
    fourslash::verify_quick_info_at(&mut s, "21", "(parameter) b: string", "");
    fourslash::verify_quick_info_at(&mut s, "22", "(parameter) args: [z: boolean]", "");
    fourslash::verify_quick_info_at(&mut s, "30", "(parameter) a: number", "");
    fourslash::verify_quick_info_at(&mut s, "31", "(parameter) b: string", "");
    fourslash::verify_quick_info_at(&mut s, "32", "(parameter) c: boolean", "");
    fourslash::verify_quick_info_at(&mut s, "33", "(parameter) args: []", "");
    fourslash::verify_quick_info_at(&mut s, "40", "(parameter) a: number", "");
    fourslash::verify_quick_info_at(
        &mut s,
        "41",
        "(parameter) args: [y: string, z: boolean]",
        "",
    );
    fourslash::verify_quick_info_at(&mut s, "50", "(parameter) a: number", "");
    fourslash::verify_quick_info_at(&mut s, "51", "(parameter) b: string", "");
    fourslash::verify_quick_info_at(&mut s, "52", "(parameter) args: [z: boolean]", "");
    fourslash::verify_quick_info_at(&mut s, "60", "(parameter) a: number", "");
    fourslash::verify_quick_info_at(&mut s, "61", "(parameter) b: string", "");
    fourslash::verify_quick_info_at(&mut s, "62", "(parameter) c: boolean", "");
    fourslash::verify_quick_info_at(&mut s, "63", "(parameter) args: []", "");
}
