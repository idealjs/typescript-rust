use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn tsx_quick_info7() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"//@Filename: file.tsx
// @jsx: preserve
// @noLib: true
declare function OverloadComponent<U>(attr: {b: U, a?: string, "ignore-prop": boolean}): JSX.Element;
declare function OverloadComponent<T, U>(attr: {b: U, a: T}): JSX.Element;
declare function OverloadComponent(): JSX.Element; // effective argument type of ` + "`" + `{}` + "`" + `, needs to be last
function Baz<T extends {b: number}, U extends {a: boolean, b:string}>(arg1: T, arg2: U) {
    let a0 = <Overloa/*1*/dComponent {...arg1} a="hello" ignore-prop />;
    let a1 = <Overloa/*2*/dComponent {...arg2} ignore-pro="hello world" />;
    let a2 = <Overloa/*3*/dComponent {...arg2} />;
    let a3 = <Overloa/*4*/dComponent {...arg1} ignore-prop />;
    let a4 = <Overloa/*5*/dComponent />;
    let a5 = <Overloa/*6*/dComponent {...arg2} ignore-prop="hello" {...arg1} />;
    let a6 = <Overloa/*7*/dComponent {...arg1} ignore-prop {...arg2} />;
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(
        &mut s,
        "1",
        "function OverloadComponent<number>(attr: {\n    b: number;\n    a?: string;\n    \"ignore-prop\": boolean;\n}): JSX.Element (+2 overloads)",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "2",
        "function OverloadComponent<boolean, string>(attr: {\n    b: string;\n    a: boolean;\n}): JSX.Element (+2 overloads)",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "3",
        "function OverloadComponent<boolean, string>(attr: {\n    b: string;\n    a: boolean;\n}): JSX.Element (+2 overloads)",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "4",
        "function OverloadComponent(): JSX.Element (+2 overloads)",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "5",
        "function OverloadComponent(): JSX.Element (+2 overloads)",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "6",
        "function OverloadComponent<boolean, never>(attr: {\n    b: never;\n    a: boolean;\n}): JSX.Element (+2 overloads)",
        "",
    );
    fourslash::verify_quick_info_at(
        &mut s,
        "7",
        "function OverloadComponent<boolean, never>(attr: {\n    b: never;\n    a: boolean;\n}): JSX.Element (+2 overloads)",
        "",
    );
}
