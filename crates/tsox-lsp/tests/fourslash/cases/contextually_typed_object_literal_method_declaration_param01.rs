use tsox_lsp::fourslash::{self, Session};

#[test]
fn contextually_typed_object_literal_method_declaration_param01() {
    let content = r#"// @noImplicitAny: true
interface A {
    numProp: number;
}

interface B  {
    strProp: string;
}

interface Foo {
    method1(arg: A): void;
    method2(arg: B): void;
}

function getFoo1(): Foo {
    return {
        method1(/*param1*/arg) {
            arg.numProp = 10;
        },
        method2(/*param2*/arg) {
            arg.strProp = "hello";
        }
    }
}"#;
    let mut s = Session::new(content);
    fourslash::verify_quick_info_at(&mut s, "param1", "(parameter) arg: A", "");
    fourslash::verify_quick_info_at(&mut s, "param2", "(parameter) arg: B", "");
}
