use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_for_contextually_typed_arrow_function_in_super_call() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"class A<T1, T2> {
    constructor(private map: (value: T1) => T2) {

    }
}

class B extends A<number, string> {
    constructor() { super(va/*1*/lue => String(va/*2*/lue.toExpone/*3*/ntial())); }
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "1", "(parameter) value: number", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(parameter) value: number", "");
    fourslash::verify_quick_info_at(
        &mut s,
        "3",
        "(method) Number.toExponential(fractionDigits?: number): string",
        "Returns a string containing a number represented in exponential notation.",
    );
}
