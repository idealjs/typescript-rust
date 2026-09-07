use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyQuickInfoAt"]
#[test]
fn quick_info_for_contextually_typed_parameters() {
    let content = r#"declare function foo1<T>(obj: T, settings: (row: T) => { value: string, func?: Function }): void;

foo1(new Error(),
    o/*1*/ => ({
        value: o.name,
        func: x => 'foo'
    })
);

declare function foo2<T>(settings: (row: T) => { value: string, func?: Function }, obj: T): void;

foo2(o/*2*/ => ({
        value: o.name,
        func: x => 'foo'
    }),
    new Error(),
);

declare function foof<T extends { name: string }, U extends keyof T>(settings: (row: T) => { value: T[U], func?: Function }, obj: T, key: U): U;

function q<T extends { name: string }>(x: T): T["name"] {
    return foof/*3*/(o => ({ value: o.name, func: x => 'foo' }), x, "name");
}

foof/*4*/(o => ({ value: o.name, func: x => 'foo' }), new Error(), "name");"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "1", "(parameter) o: Error", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "2", "(parameter) o: Error", "")
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "3", "function foof<T, \"name\">(settings: (row: T) => {\n    value: T[\"name
    fourslash::unsupported("VerifyQuickInfoAt"); // f.VerifyQuickInfoAt(t, "4", "function foof<Error, \"name\">(settings: (row: Error) => {\n    value: 
}
