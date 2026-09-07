use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCodeFixAll"]
#[test]
fn code_fix_class_implement_interface_callback() {
    let content = r#"interface IFoo1 {
    parse(reviver: () => any): void;
}

class Foo1 implements IFoo1 {
}

interface IFoo2 {
    parse(reviver: { (): any }): void;
}

class Foo2 implements IFoo2 {
}

interface IFoo3 {
    parse(reviver: new () => any): void;
}

class Foo3 implements IFoo3 {
}

interface IFoo4 {
    parse(reviver: { new (): any }): void;
}

class Foo4 implements IFoo4 {
}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCodeFixAll"); // f.VerifyCodeFixAll(t, fourslash.VerifyCodeFixAllOptions{
}
