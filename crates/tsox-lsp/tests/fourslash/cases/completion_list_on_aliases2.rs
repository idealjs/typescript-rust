use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_list_on_aliases2() {
    let content = r#"// @lib: es5
namespace M {
    export interface I { }
    export class C {
        static property;
    }
    export enum E {
        value = 0
    }
    export namespace N {
        export var v;
    }
    export var V = 0;
    export function F() { }
    export import A = M;
}

import m = M;
import c = M.C;
import e = M.E;
import n = M.N;
import v = M.V;
import f = M.F;
import a = M.A;

m./*1*/;
var tmp: m./*1Type*/;
c./*2*/;
e./*3*/;
n./*4*/;
v./*5*/;
f./*6*/;
a./*7*/;
var tmp2: a./*7Type*/;"#;
    let mut s = Session::new_for_test("completionListOnAliases2", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "7"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, []string{"1Type", "7Type"}, &fourslash.CompletionsExpectedList{
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::verify_completions_exact_at(&mut s, Some("3"), &["value"]);
    fourslash::verify_completions_exact_at(&mut s, Some("4"), &["v"]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("5"), &["toFixed"], &[]);
    fourslash::verify_completions_include_exclude_at(&mut s, Some("6"), &["call"], &[]);
}
