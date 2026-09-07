use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_in_optional_chain() {
    let content = r#"// @strict: true
interface A {
  arr: string[];
}

function test(a?: A): string {
  return a?.ar/*1*/r.length ? "A" : "B";
}

interface Foo { bar: { baz: string } };
declare const foo: Foo | undefined;

if (foo?.b/*2*/ar.b/*3*/az) {}

interface Foo2 { bar?: { baz: { qwe: string } } };
declare const foo2: Foo2;

if (foo2.b/*4*/ar?.b/*5*/az.q/*6*/we) {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(property) A.arr: string[]", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(property) Foo.bar: {\n    baz: string;\n}", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "(property) baz: string | undefined", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "(property) Foo2.bar?: {\n    baz: {\n        qwe: string;\n    };\n} | 
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "(property) baz: {\n    qwe: string;\n}", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6", "(property) qwe: string | undefined", "")
}
