use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCodeFix"]
#[test]
fn code_fix_class_implement_interface_computed_property_name_well_known_symbols() {
    let content = r#"// @strict: false
// @lib: es2017
interface I<Species> {
    [Symbol.hasInstance](o: any): boolean;
    [Symbol.isConcatSpreadable]: boolean;
    [Symbol.iterator](): any;
    [Symbol.match]: boolean;
    [Symbol.replace](...args);
    [Symbol.search](str: string): number;
    [Symbol.species](): Species;
    [Symbol.split](str: string, limit?: number): string[];
    [Symbol.toPrimitive](hint: "number"): number;
    [Symbol.toPrimitive](hint: "default"): number;
    [Symbol.toPrimitive](hint: "string"): string;
    [Symbol.toStringTag]: string;
    [Symbol.unscopables]: any;
}
class C implements I<number> {}"#;
    let mut s = Session::new_for_test("codeFixClassImplementInterfaceComputedPropertyNameWellKnownSymbols", content);
    fourslash::unsupported("VerifyCodeFix"); // f.VerifyCodeFix(t, fourslash.VerifyCodeFixOptions{
}
