use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn overload_quick_info() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"function Foo(a: string, b: number, c: boolean);
function Foo(a: any, name: string, age: number);
function Foo(fred: any[], name: string, age: number);
function Foo(fred: any[  ] , name: string[], age: number);
function Foo(fred: any[], name: string[], age: number[]);
function Foo(fred:         any, name: string[], age: number[]); // Extraneous spaces should get removed
function Foo(fred: any, name: boolean, age: number[]);
function Foo(dave: boolean, name: string);
function Foo(fred: any, mandy: {(): number}, age: number[]);    // Embedded interface will get converted to shorthand notation, () => 
function Foo(fred: any, name: string, age: { });
function Foo(fred: any, name: string, age: number[]);
function Foo(test: string, name, age: number);
function Foo();
function Foo(x?: any, y?: any, z?: any) {
}
Fo/**/o();"#;
    let mut s = Session::new_for_test("overloadQuickInfo", content);
    fourslash::verify_quick_info_at(&mut s, "", "function Foo(): any (+12 overloads)", "");
}
