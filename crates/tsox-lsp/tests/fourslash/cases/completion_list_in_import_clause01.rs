use tsox_lsp::fourslash::Session;


#[test]
fn completion_list_in_import_clause01() {
    let content = r#"// @Filename: m1.ts
export var foo: number = 1;
export function bar() { return 10; }
export function baz() { return 10; }
// @Filename: m2.ts
import {/*1*/, /*2*/ from "./m1"
import {/*3*/} from "./m1"
import {foo,/*4*/ from "./m1"
import {bar as /*5*/, /*6*/ from "./m1"
import {foo, bar, baz as b,/*7*/} from "./m1"
import { type /*8*/ } from "./m1";
import { type b/*9*/ } from "./m1";"#;
    let _s = Session::new_for_test("completionListInImportClause01", content);
    // TODO: f.VerifyCompletions(t, []string{"8", "9"}, &fourslash.CompletionsExpectedList{
    // TODO: }
}
