use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
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
    let mut s = Session::new_for_test("navigationItemsExportDefaultExpression2", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
