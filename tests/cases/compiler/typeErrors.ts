// @strict: true
// @noImplicitAny: true

// Error: 'x' implicitly has an 'any' type
function foo(x) {
    return x + 1;
}

// Error: Type 'string' is not assignable to type 'number'
const y: number = "hello";
