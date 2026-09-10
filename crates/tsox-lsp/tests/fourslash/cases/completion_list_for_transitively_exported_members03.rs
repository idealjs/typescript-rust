use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_for_transitively_exported_members03() {
    let content = r#"// @Filename: A.ts
export interface I1 { one: number }
export interface I2 { two: string }
export type I1_OR_I2 = I1 | I2;

export class C1 {
    one: string;
}

export namespace Inner {
    export interface I3 {
        three: boolean
    }

    export var varVar = 100;
    export let letVar = 200;
    export const constVar = 300;
}
// @Filename: B.ts
export var bVar = "bee!";
// @Filename: C.ts
export var cVar = "see!";
export * from "./A";
export * from "./B"
// @Filename: D.ts
import * as c from "./C";
var x: c./**/"#;
    let mut s = Session::new_for_test("completionListForTransitivelyExportedMembers03", content);
    fourslash::verify_completions_include_exclude_at(&mut s, Some(""), &["I1", "I2", "I1_OR_I2", "C1"], &[]);
}
