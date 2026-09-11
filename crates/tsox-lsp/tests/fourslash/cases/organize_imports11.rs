use tsox_lsp::fourslash::{self, Session};


#[test]
fn organize_imports11() {
    let content = r#"// @Filename: /test.ts
import { TypeA, TypeB, TypeC, UnreferencedType } from './my-types';

/**
 * MyClass {@link TypeA}
 */
export class MyClass {

  /**
   * Some Property {@link TypeB}
   */
  public something;

  /**
   * Some function {@link TypeC}
   */
  public myMethod() {

    /**
     * Some lambda function {@link TypeC}
     */
    const someFunction = () => {
      return '';
    }
    someFunction();
  }
}
// @Filename: /my-types.ts
 export type TypeA = string;
 export class TypeB { }
 export type TypeC = () => string;"#;
    let mut s = Session::new_for_test("organizeImports11", content);
    // TODO: f.VerifyOrganizeImports(t,
}
