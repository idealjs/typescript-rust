use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_mapped_type_recursive_inference() {
    let content = r#"// @Filename: test.ts
interface A { a: A }
declare let a: A;
type Deep<T> = { [K in keyof T]: Deep<T[K]> }
declare function foo<T>(deep: Deep<T>): T;
const out/*1*/ = foo/*2*/(a);
out.a/*3*/
out.a.a/*4*/
out.a.a.a.a.a.a.a/*5*/

interface B { [s: string]: B }
declare let b: B;
const oub/*6*/ = foo/*7*/(b);
oub.b/*8*/
oub.b.b/*9*/
oub.b.a.n.a.n.a/*10*/"#;
    let mut s = Session::new_for_test("quickInfoMappedTypeRecursiveInference", content);
    fourslash::verify_quick_info_at(&mut s, "1", "const out: {\n    a: {\n        a: ...;\n    };\n}", "");
    fourslash::verify_quick_info_at(&mut s, "2", "function foo<{\n    a: {\n        a: ...;\n    };\n}>(deep: Deep<{\n    a: {\n        a: ...;\n    };\n}>): {\n    a: {\n        a: ...;\n    };\n}", "");
    fourslash::verify_quick_info_at(&mut s, "3", "(property) a: {\n    a: {\n        a: ...;\n    };\n}", "");
    fourslash::verify_quick_info_at(&mut s, "4", "(property) a: {\n    a: {\n        a: ...;\n    };\n}", "");
    fourslash::verify_quick_info_at(&mut s, "5", "(property) a: {\n    a: {\n        a: ...;\n    };\n}", "");
    fourslash::verify_quick_info_at(&mut s, "6", "const oub: {\n    [x: string]: ...;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "7", "function foo<{\n    [x: string]: ...;\n}>(deep: Deep<{\n    [x: string]: ...;\n}>): {\n    [x: string]: ...;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "8", "{\n    [x: string]: ...;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "9", "{\n    [x: string]: ...;\n}", "");
    fourslash::verify_quick_info_at(&mut s, "10", "{\n    [x: string]: ...;\n}", "");
}
