use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn underscore_typings01() {
    let content = r#"interface Iterator_<T, U> {
    (value: T, index: any, list: any): U;
}

interface WrappedArray<T> {
    map<U>(iterator: Iterator_<T, U>, context?: any): U[];
}

interface Underscore {
    <T>(list: T[]): WrappedArray<T>;
    map<T, U>(list: T[], iterator: Iterator_<T, U>, context?: any): U[];
}

declare var _: Underscore;

var a: string[];
var /*1*/b = _.map(a, /*2*/x => x.length);    // Was typed any[], should be number[]
var /*3*/c = _(a).map(/*4*/x => x.length);
var /*5*/d = a.map(/*6*/x => x.length);

var aa: any[];
var /*7*/bb = _.map(aa, /*8*/x => x.length);
var /*9*/cc = _(aa).map(/*10*/x => x.length);
var /*11*/dd = aa.map(/*12*/x => x.length);

var e = a.map(x => x./*13*/"#;
    let mut s = Session::new_for_test("underscoreTypings01", content);
    fourslash::verify_quick_info_at(&mut s, "1", "var b: number[]", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(parameter) x: string", "");
    fourslash::verify_quick_info_at(&mut s, "3", "var c: number[]", "");
    fourslash::verify_quick_info_at(&mut s, "4", "(parameter) x: string", "");
    fourslash::verify_quick_info_at(&mut s, "5", "var d: number[]", "");
    fourslash::verify_quick_info_at(&mut s, "6", "(parameter) x: string", "");
    fourslash::verify_quick_info_at(&mut s, "7", "var bb: any[]", "");
    fourslash::verify_quick_info_at(&mut s, "8", "(parameter) x: any", "");
    fourslash::verify_quick_info_at(&mut s, "9", "var cc: any[]", "");
    fourslash::verify_quick_info_at(&mut s, "10", "(parameter) x: any", "");
    fourslash::verify_quick_info_at(&mut s, "11", "var dd: any[]", "");
    fourslash::verify_quick_info_at(&mut s, "12", "(parameter) x: any", "");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "13", &fourslash.CompletionsExpectedList{
}
