use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "4", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "5", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "6", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "7", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "8", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "9", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "10", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "11", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "12", &fourslash.CompletionsExpectedList{
}
