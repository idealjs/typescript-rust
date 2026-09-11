use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_type_member_completions() {
    let content = r#"// @Filename: /ns.ts
export namespace Foo {
    export namespace Bar {
        export class Baz {}
        export interface Bat {}
        export const a: number;
        const b: string;
    }
}
// @Filename: /top.ts
export interface Bat {}
export const a: number;
// @Filename: /equals.ts
class Foo {
 public static bar: string;
 private static baz: number;
}
export = Foo;
// @Filename: /usage1.ts
type A = typeof import("./ns")./*1*/
// @Filename: /usage2.ts
type B = typeof import("./ns").Foo./*2*/
// @Filename: /usage3.ts
type C = typeof import("./ns").Foo.Bar./*3*/
// @Filename: /usage4.ts
type D = import("./ns")./*4*/
// @Filename: /usage5.ts
type E = import("./ns").Foo./*5*/
// @Filename: /usage6.ts
type F = import("./ns").Foo.Bar./*6*/
// @Filename: /usage7.ts
type G = typeof import("./top")./*7*/
// @Filename: /usage8.ts
type H = import("./top")./*8*/
// @Filename: /usage9.ts
type H = typeof import("./equals")./*9*/"#;
    let mut s = Session::new_for_test("importTypeMemberCompletions", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["Foo"]);
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["Bar"]);
    fourslash::verify_completions_exact_at(&mut s, Some("3"), &["a", "Baz"]);
    fourslash::verify_completions_exact_at(&mut s, Some("4"), &["Foo"]);
    fourslash::verify_completions_exact_at(&mut s, Some("5"), &["Bar"]);
    fourslash::verify_completions_exact_at(&mut s, Some("6"), &["Bat", "Baz"]);
    fourslash::verify_completions_exact_at(&mut s, Some("7"), &["a"]);
    fourslash::verify_completions_exact_at(&mut s, Some("8"), &["Bat"]);
    fourslash::verify_completions_exact_at(&mut s, Some("9"), &["bar", "prototype"]);
}
