use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineFindAllReferences"]
#[test]
fn find_all_refs_js_doc_import_tag4() {
    let content = r#"// @checkJs: true
// @Filename: /component.js
export class Component {
  constructor() {
    this.id_ = Math.random();
  }
  id() {
    return this.id_;
  }
}
// @Filename: /spatial-navigation.js
/** @import * as C from './component.js' */

export class SpatialNavigation {
  /**
   * @param {C.Component} component
   */
  add(component) {}
}
// @Filename: /player.js
import * as C from './component.js';

/**
 * @extends C/*1*/.Component
 */
export class Player extends Component {}"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
