use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn jsx_attribute_snippet_completion_after_type_args() {
    let content = r#"// @strict: false
//@Filename: file.tsx
declare const React: any;

namespace JSX {
    export interface IntrinsicElements {
        div: any;
    }
}

function GenericElement<T>(props: {xyz?: T}) {
    return <></>
}

function fn1() {
    return <div>
        <GenericElement<number> /*1*/ />
    </div>
}

function fn2() {
    return <>
        <GenericElement<number> /*2*/ />
    </>
}
function fn3() {
    return <div>
        <GenericElement<number> /*3*/ ></GenericElement>
    </div>
}

function fn4() {
    return <>
        <GenericElement<number> /*4*/ ></GenericElement>
    </>
}"#;
    let mut s = Session::new_with_capabilities(content, None);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
