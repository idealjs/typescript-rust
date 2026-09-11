use tsox_lsp::fourslash::{self, Session};


#[test]
fn navigation_items_export_equals_expression2() {
    let content = r#"export const foo = {
  foo: {},
};

export = {
  foo: {},
};

export = {
  foo: {},
};

type Type = typeof foo;

export = {
  foo: {},
} as Type;

export = {
  foo: {},
} satisfies Type;

export = (class {
  prop = 42;
});

export = (class Cls {
  prop = 42;
});"#;
    let mut s = Session::new_for_test("navigationItemsExportEqualsExpression2", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
