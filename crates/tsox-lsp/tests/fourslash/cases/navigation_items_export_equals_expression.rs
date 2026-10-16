use tsox_lsp::fourslash::Session;


#[test]
fn navigation_items_export_equals_expression() {
    let content = r#"export = function () {}
export = function () {
    return class Foo {
    }
}

export = () => ""
export = () => {
    return class Foo {
    }
}

export = function f1() {}
export = function f2() {
    return class Foo {
    }
}

const abc = 12;
export = abc;
export = class AB {}
export = {
    a: 1,
    b: 1,
    c: {
        d: 1
    }
}

function foo(props: { x: number; y: number }) {}
export = foo({ x: 1, y: 1 });"#;
    let _s = Session::new_for_test("navigationItemsExportEqualsExpression", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
