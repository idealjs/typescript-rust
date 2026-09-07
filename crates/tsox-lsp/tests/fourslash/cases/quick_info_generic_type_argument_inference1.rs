use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn quick_info_generic_type_argument_inference1() {
    let content = r#"// @strict: false
namespace Underscore {
    export interface Iterator<T, U> {
        (value: T, index: any, list: any): U;
    }

    export interface Static {
        all<T>(list: T[], iterator?: Iterator<T, boolean>, context?: any): T;
        identity<T>(value: T): T;
    }
}

declare var _: Underscore.Static;
var /*1*/r = _./*11*/all([true, 1, null, 'yes'], x => !x);
var /*2*/r2 = _./*21*/all([true], _.identity);
var /*3*/r3 = _./*31*/all([], _.identity);
var /*4*/r4 = _./*41*/all([<any>true], _.identity);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var r: string | number | boolean", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "11", "(method) Underscore.Static.all<string | number | boolean>(list: (strin
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var r2: boolean", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "21", "(method) Underscore.Static.all<boolean>(list: boolean[], iterator?: Un
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "var r3: any", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "31", "(method) Underscore.Static.all<any>(list: any[], iterator?: Underscore
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "var r4: any", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "41", "(method) Underscore.Static.all<any>(list: any[], iterator?: Underscore
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
