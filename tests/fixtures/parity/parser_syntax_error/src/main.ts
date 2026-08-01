// Various syntax errors to test parser diagnostics parity
// TS1003: Identifier expected
const = 42;

// TS1005: ')' expected
function foo(a, b {
  return a + b;
}

// TS1109: Expression expected
let x = ;

// TS1136: Property signature expected
interface Foo {
 ;
}
