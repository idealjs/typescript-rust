use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn quick_info_on_internal_aliases() {
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
    let mut s = Session::new_for_test("quickInfoOnInternalAliases", content);
    fourslash::verify_quick_info_at(&mut s, "1", "class m1.m2.c", "class comment;");
    fourslash::verify_quick_info_at(&mut s, "2", "(alias) class internalAlias\nimport internalAlias = m1.m2.c", "This is on import declaration");
    fourslash::verify_quick_info_at(&mut s, "3", "class m1.m2.c", "class comment;");
    fourslash::verify_quick_info_at(&mut s, "4", "var newVar: internalAlias", "");
    fourslash::verify_quick_info_at(&mut s, "5", "(alias) new internalAlias(): internalAlias\nimport internalAlias = m1.m2.c", "This is on import declaration");
    fourslash::verify_quick_info_at(&mut s, "6", "var anotherAliasVar: typeof internalAlias", "");
    fourslash::verify_quick_info_at(&mut s, "7", "(alias) class internalAlias\nimport internalAlias = m1.m2.c", "This is on import declaration");
    fourslash::verify_quick_info_at(&mut s, "8", "(alias) function internalFoo(): void\nimport internalFoo = m1.foo", "");
    fourslash::verify_quick_info_at(&mut s, "9", "function m1.foo(): void", "");
    fourslash::verify_quick_info_at(&mut s, "10", "var callVar: void", "");
    fourslash::verify_quick_info_at(&mut s, "11", "(alias) internalFoo(): void\nimport internalFoo = m1.foo", "");
    fourslash::verify_quick_info_at(&mut s, "12", "var anotherAliasFoo: () => void", "");
    fourslash::verify_quick_info_at(&mut s, "13", "(alias) function internalFoo(): void\nimport internalFoo = m1.foo", "");
}
