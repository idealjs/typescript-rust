use tsox_lsp::fourslash::{self, Session};


#[test]
fn hover_mapped_type_property_js_doc() {
    let content = r#"
// @filename: a.ts
export declare const A: Readonly<{
  /**
   * x prop
   */
  readonly X: 200;

  /**
   * y prop
   */
  readonly Y: 201;
}>;

A.X/*1*/;

// @filename: b.ts
import { A } from './a';

A.X/*2*/;
"#;
    let mut s = Session::new_for_test("hoverMappedTypePropertyJSDoc", content);
    // TODO: f.VerifyBaselineHover(t)
}

#[test]
fn hover_mapped_type_without_property_type() {
    let content = r#"
declare function uhoh/*1*/<T>(x: { [K in keyof T] }): void;
"#;
    let mut s = Session::new_for_test("hoverMappedTypeWithoutPropertyType", content);
    // TODO: f.VerifyBaselineHover(t)
}
