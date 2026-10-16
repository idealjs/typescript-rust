use tsox_lsp::fourslash::{self, Session};


#[test]
fn format_multiline_types_with_mapped() {
    let content = r#"type Z = 'z'
type A = {
  a: 'a'
} | {
      [index in Z]: string
  }
type B = {
  b: 'b'
} & {
      [index in Z]: string
  }

const c = {
  c: 'c'
} as const satisfies {
    [index in Z]: string
  }

const d = {
  d: 'd'
} as const satisfies {
  [index: string]: string
}

const e = {
  e: 'e'
} satisfies {
    [index in Z]: string
  }

const f = {
  f: 'f'
} satisfies {
  [index: string]: string
}"#;
    let mut s = Session::new_for_test("formatMultilineTypesWithMapped", content);
    fourslash::format_document(&mut s, "");
    fourslash::verify_current_file_content(&mut s, r#"type Z = 'z'
type A = {
    a: 'a'
} | {
    [index in Z]: string
}
type B = {
    b: 'b'
} & {
    [index in Z]: string
}

const c = {
    c: 'c'
} as const satisfies {
    [index in Z]: string
}

const d = {
    d: 'd'
} as const satisfies {
    [index: string]: string
}

const e = {
    e: 'e'
} satisfies {
    [index in Z]: string
}

const f = {
    f: 'f'
} satisfies {
    [index: string]: string
}"#);
}
