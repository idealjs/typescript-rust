// JSX (in .jsx) parsing test cases
// Tests JSX without TypeScript type annotations.

const React = require("react");

// Function component (no type annotations)
function Welcome(props) {
  return <h1>Hello, {props.name}</h1>;
}

// Arrow function component
const Goodbye = ({ name }) => (
  <div className="goodbye">
    <p>See you later, {name}!</p>
  </div>
);

// Fragment
const App = () => (
  <>
    <Welcome name="Alice" />
    <Goodbye name="Bob" />
    <input type="text" value="test" onChange={(e) => console.log(e.target.value)} />
  </>
);

// Conditional rendering
function List({ items }) {
  return (
    <ul>
      {items.map((item, i) => (
        <li key={i}>{item}</li>
      ))}
    </ul>
  );
}

module.exports = { Welcome, Goodbye, App, List };
