use tsox_lsp::fourslash::{self, Session};


#[test]
fn find_all_refs_js_doc_import_tag2() {
    let content = r#"// @checkJs: true
// @Filename: /component.js
export default class Component {
  constructor() {
    this.id_ = Math.random();
  }
  id() {
    return this.id_;
  }
}
// @Filename: /spatial-navigation.js
/** @import Component from './component.js' */

export class SpatialNavigation {
  /**
   * @param {Component} component
   */
  add(component) {}
}
// @Filename: /player.js
import Component from './component.js';

/**
 * @extends Component/*1*/
 */
export class Player extends Component {}"#;
    let mut s = Session::new_for_test("findAllRefsJsDocImportTag2", content);
    // TODO: f.VerifyBaselineFindAllReferences(t, "1")
}
