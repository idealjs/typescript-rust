use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToImplementation"]
#[test]
fn go_to_implementation_interface_07() {
    let content = r#"interface Fo/*interface_definition*/o {
    hello (): void;
}

interface Bar {
    hello (): void;
}

let x1: Foo            = [|{ hello ()          { /**typeReference*/ } }|];
let x2: () => Foo      = [|(() => { hello ()   { /**functionType*/} })|];
let x3: Foo | Bar      = [|{ hello ()          { /**unionType*/} }|];
let x4: Foo & (Foo & Bar)      = [|{ hello ()          { /**intersectionType*/} }|];
let x5: [Foo]          = [|[{ hello ()         { /**tupleType*/} }]|];
let x6: (Foo)          = [|{ hello ()          { /**parenthesizedType*/} }|];
let x7: (new() => Foo) = [|class { hello ()    { /**constructorType*/} }|];
let x8: Foo[]          = [|[{ hello ()         { /**arrayType*/} }]|];
let x9: { y: Foo }     = [|{ y: { hello ()     { /**typeLiteral*/} } }|];
let x10 = [|{|"parts": ["(","anonymous local class",")"], "kind": "local class"|}class implements Foo { hello() {} }|]
let x11 = class [|{|"parts": ["(","local class",")"," ","C"], "kind": "local class"|}C|] implements Foo { hello() {} }

// Should not do anything for type predicates
function isFoo(a: any): a is Foo {
    return true;
}"#;
    let mut s = Session::new_for_test("goToImplementationInterface_07", content);
    fourslash::unsupported("VerifyBaselineGoToImplementation"); // f.VerifyBaselineGoToImplementation(t, "interface_definition")
}
