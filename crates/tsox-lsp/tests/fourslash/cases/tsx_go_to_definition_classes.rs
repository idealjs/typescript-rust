use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn tsx_go_to_definition_classes() {
    let content = r#"//@Filename: file.tsx
declare namespace JSX {
    interface Element { }
    interface IntrinsicElements { }
    interface ElementAttributesProperty { props; }
}
class /*ct*/MyClass {
    props: {
        /*pt*/foo: string;
    }
}
var x = <[|My/*c*/Class|] />;
var y = <MyClass [|f/*p*/oo|]= 'hello' />;
var z = <[|MyCl/*w*/ass|] wrong= 'hello' />;"#;
    let mut s = Session::new_for_test("tsxGoToDefinitionClasses", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "c", "p", "w")
}
