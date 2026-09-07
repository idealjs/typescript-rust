use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_internal_aliases() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"/** Module comment*/
export namespace m1 {
    /** m2 comments*/
    export namespace m2 {
        /** class comment;*/
        export class /*1*/c {
        };
    }
    export function foo() {
    }
}
/**This is on import declaration*/
import /*2*/internalAlias = m1.m2./*3*/c;
var /*4*/newVar = new /*5*/internalAlias();
var /*6*/anotherAliasVar = /*7*/internalAlias;
import /*8*/internalFoo = m1./*9*/foo;
var /*10*/callVar = /*11*/internalFoo();
var /*12*/anotherAliasFoo = /*13*/internalFoo;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "class m1.m2.c", "class comment;")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(alias) class internalAlias\nimport internalAlias = m1.m2.c", "This is 
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "class m1.m2.c", "class comment;")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "var newVar: internalAlias", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "5", "(alias) new internalAlias(): internalAlias\nimport internalAlias = m1.m
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "6", "var anotherAliasVar: typeof internalAlias", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "7", "(alias) class internalAlias\nimport internalAlias = m1.m2.c", "This is 
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "8", "(alias) function internalFoo(): void\nimport internalFoo = m1.foo", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "9", "function m1.foo(): void", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "10", "var callVar: void", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "11", "(alias) internalFoo(): void\nimport internalFoo = m1.foo", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "12", "var anotherAliasFoo: () => void", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "13", "(alias) function internalFoo(): void\nimport internalFoo = m1.foo", ""
}
