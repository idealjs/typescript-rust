use tsox_lsp::fourslash::Session;


#[test]
fn navigation_items_export_default_expression() {
    let content = r#"export default function () {}
export default function () {
    return class Foo {
    }
}

export default () => ""
export default () => {
    return class Foo {
    }
}

export default function f1() {}
export default function f2() {
    return class Foo {
    }
}

const abc = 12;
export default abc;
export default class AB {}
export default {
    a: 1,
    b: 1,
    c: {
        d: 1
    }
}

function foo(props: { x: number; y: number }) {}
export default foo({ x: 1, y: 1 });"#;
    let _s = Session::new_for_test("navigationItemsExportDefaultExpression", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
