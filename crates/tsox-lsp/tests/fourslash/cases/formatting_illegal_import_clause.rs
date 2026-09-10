use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.FormatDocument"]
#[test]
fn formatting_illegal_import_clause() {
    let content = r#"var expect = require('expect.js');
import React   from 'react'/*1*/;
import { mount } from 'enzyme';
require('../setup');
var Amount = require('../../src/js/components/amount');
describe('<Failed />', () => {
  var history
  beforeEach(() => {
    history = createMemoryHistory();
    sinon.spy(history, 'pushState');
  });
  afterEach(() => {
  })
  it('redirects to order summary', () => {
  });
});"#;
    let mut s = Session::new_for_test("formattingIllegalImportClause", content);
    fourslash::unsupported("FormatDocument"); // f.FormatDocument(t, "")
    fourslash::go_to_marker(&mut s, "1");
    fourslash::verify_current_line_content(&mut s, r#"import React from 'react';"#);
}
