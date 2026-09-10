use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("quickInfoForContextuallyTypedParameters", content);
    fourslash::verify_quick_info_at(&mut s, "1", "(parameter) o: Error", "");
    fourslash::verify_quick_info_at(&mut s, "2", "(parameter) o: Error", "");
    fourslash::verify_quick_info_at(&mut s, "3", "function foof<T, \"name\">(settings: (row: T) => {\n    value: T[\"name\"];\n    func?: Function;\n}, obj: T, key: \"name\"): \"name\"", "");
    fourslash::verify_quick_info_at(&mut s, "4", "function foof<Error, \"name\">(settings: (row: Error) => {\n    value: string;\n    func?: Function;\n}, obj: Error, key: \"name\"): \"name\"", "");
}
