use tsox_lsp::fourslash::{self, Session};


#[test]
fn destructured_interface_js_doc() {
    let content = r#"
interface FooBar {
    /** foo comment */
    foo: number;
    /** bar comment */
    bar: string;
    /** baz comment */
    baz: string;
}

declare const fubar: FooBar;

const {/*1*/foo, /*2*/bar, /*3*/baz: /*4*/biz} = fubar;
"#;
    let mut s = Session::new_for_test("destructuredInterfaceJSDoc", content);
    // TODO: f.VerifyBaselineHover(t)
}

#[test]
fn destructured_interface_js_doc_with_rename() {
    let content = r#"
interface FooBar {
    /** foo comment */
    foo: number;
    /** bar comment */
    bar: string;
}

declare const fubar: FooBar;

const {foo: /*1*/myFoo, bar: /*2*/myBar} = fubar;
"#;
    let mut s = Session::new_for_test("destructuredInterfaceJSDocWithRename", content);
    // TODO: f.VerifyBaselineHover(t)
}

#[test]
fn destructured_with_own_js_doc() {
    let content = r#"
interface Foo {
    /** This is bar from the interface */
    bar: string;
    /** This is baz from the interface */
    baz: number;
}

declare var foo: Foo;

/** Comment on the variable statement. */
const {
    /** Comment on bar destructuring. */ /*1*/bar,
    /** Comment on baz destructuring. */ /*2*/baz
} = foo;
"#;
    let mut s = Session::new_for_test("destructuredWithOwnJSDoc", content);
    // TODO: f.VerifyBaselineHover(t)
}
