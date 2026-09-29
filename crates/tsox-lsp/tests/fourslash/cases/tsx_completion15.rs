use tsox_lsp::fourslash::{self, Session};


#[test]
fn tsx_completion15() {
    let content = r#"//@module: commonjs
//@jsx: preserve
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements {
    }
    interface ElementAttributesProperty { props; }
}
//@Filename: exporter.tsx
export namespace M {
   export declare function SFCComp(props: { Three: number; Four: string }): JSX.Element;
}
//@Filename: file.tsx
import * as Exp from './exporter';
var x1  = <Exp.M.SFCComp></[|/*1*/|]>;
var x2  = <Exp.M.SFCComp></[|Exp./*2*/|]>;
var x3  = <Exp.M.SFCComp></[|Exp.M./*3*/|]>;
var x4  = <Exp.M.SFCComp></[|Exp.M.SFCComp/*4*/|]
var x5  = <Exp.M.SFCComp></[|Exp.M.SFCComp/*5*/|]>;
var x6  = <Exp.M.SFCComp></      [|Exp./*6*/|]>;
var x7  = <Exp.M.SFCComp></[|/*7*/Exp.M.SFCComp|]>;
var x8  = <Exp.M.SFCComp></[|Exp/*8*/|]>;
var x9  = <Exp.M.SFCComp></[|Exp.M./*9*/|]>;
var x10 = <Exp.M.SFCComp></      [|/*10*/Exp.M.Foo.Bar.Baz.Wut|]>;
var x11 = <Exp.M.SFCComp></[|Exp./*11*/M.SFCComp|]>;
var x12 = <Exp.M.SFCComp><div><span /></div></[|Exp.M./*12*/SFCComp|]>;"#;
    let mut s = Session::new_for_test("tsxCompletion15", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["Exp.M.SFCComp"]);
    fourslash::verify_completions_exact_at(&mut s, Some("2"), &["Exp.M.SFCComp"]);
    fourslash::verify_completions_exact_at(&mut s, Some("3"), &["Exp.M.SFCComp"]);
    fourslash::verify_completions_exact_at(&mut s, Some("4"), &["Exp.M.SFCComp>"]);
    fourslash::verify_completions_exact_at(&mut s, Some("5"), &["Exp.M.SFCComp"]);
    fourslash::verify_completions_exact_at(&mut s, Some("6"), &["Exp.M.SFCComp"]);
    fourslash::verify_completions_exact_at(&mut s, Some("7"), &["Exp.M.SFCComp"]);
    fourslash::verify_completions_exact_at(&mut s, Some("8"), &["Exp.M.SFCComp"]);
    fourslash::verify_completions_exact_at(&mut s, Some("9"), &["Exp.M.SFCComp"]);
    fourslash::verify_completions_exact_at(&mut s, Some("10"), &["Exp.M.SFCComp"]);
    fourslash::verify_completions_exact_at(&mut s, Some("11"), &["Exp.M.SFCComp"]);
    fourslash::verify_completions_exact_at(&mut s, Some("12"), &["Exp.M.SFCComp"]);
}
