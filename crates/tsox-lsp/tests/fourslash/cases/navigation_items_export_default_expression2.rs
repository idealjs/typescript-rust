use tsox_lsp::fourslash::Session;


#[test]
fn navigation_items_export_default_expression2() {
    let content = r#"export const foo = {
  foo: {},
};

export default {
  foo: {},
};

export default {
  foo: {},
};

type Type = typeof foo;

export default {
  foo: {},
} as Type;

export default {
  foo: {},
} satisfies Type;

export default (class {
  prop = 42;
});

export default (class Cls {
  prop = 42;
});"#;
    let _s = Session::new_for_test("navigationItemsExportDefaultExpression2", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
