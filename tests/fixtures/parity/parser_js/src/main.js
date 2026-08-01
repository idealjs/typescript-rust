// JavaScript parsing test cases (no type annotations)
// Tests common JS patterns. Kept simple to focus on parser parity.

// Function declaration with hoisting
function greet(name) {
  return "Hello, " + name;
}

// Constructor function + prototype
function Animal(name) {
  this.name = name;
}
Animal.prototype.speak = function () {
  return this.name + " makes a sound";
};

// var hoisting pattern
var x = 1;
var x = 2; // legal with var

// Template literal
var message = greet("World") + "! You have 3 items.";

// Array and object
var arr = [1, 2, 3, 4];
var first = arr[0];
var obj = { greet: greet, Animal: Animal };
