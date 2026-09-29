use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_for_generic_tagged_template_expression() {
    let content = r#"interface T1 {}
class T2 {}
type T3 = "a" | "b";

declare function foo<T>(strings: TemplateStringsArray, ...values: T[]): void;

/*1*/foo<number>``;
/*2*/foo<string | number>``;
/*3*/foo<{ a: number }>``;
/*4*/foo<T1>``;
/*5*/foo<T2>``;
/*6*/foo<T3>``;
/*7*/foo``;"#;
    let mut s = Session::new_for_test("quickInfoForGenericTaggedTemplateExpression", content);
    fourslash::verify_quick_info_at(&mut s, "1", "function foo<number>(strings: TemplateStringsArray, ...values: number[]): void", "");
    fourslash::verify_quick_info_at(&mut s, "2", "function foo<string | number>(strings: TemplateStringsArray, ...values: (string | number)[]): void", "");
    fourslash::verify_quick_info_at(&mut s, "3", "function foo<{\n    a: number;\n}>(strings: TemplateStringsArray, ...values: {\n    a: number;\n}[]): void", "");
    fourslash::verify_quick_info_at(&mut s, "4", "function foo<T1>(strings: TemplateStringsArray, ...values: T1[]): void", "");
    fourslash::verify_quick_info_at(&mut s, "5", "function foo<T2>(strings: TemplateStringsArray, ...values: T2[]): void", "");
    fourslash::verify_quick_info_at(&mut s, "6", "function foo<T3>(strings: TemplateStringsArray, ...values: T3[]): void", "");
    fourslash::verify_quick_info_at(&mut s, "7", "function foo<unknown>(strings: TemplateStringsArray, ...values: unknown[]): void", "");
}
