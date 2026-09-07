use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_of_string_property_names1() {
    let content = r#"interface foo {
    "foo bar": string;
}
var f: foo;
var /*1*/r = f['foo bar'];
class bar {
    'hello world': number;
    '1': string;
    constructor() {
        bar['hello world'] = 3;
    }
}
var b: bar;
var /*2*/r2 = b["hello world"];
var /*3*/r4 = b['1'];
var /*4*/r5 = b[1];"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "var r: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "var r2: number", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "var r4: string", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "var r5: string", "")
}
