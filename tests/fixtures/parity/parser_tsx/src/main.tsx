// TSX JSX parsing test cases
interface Props {
  name: string;
  age: number;
}

function Greeting({ name, age }: Props) {
  return (
    <div className="greeting">
      <h1>Hello, {name}!</h1>
      <p>You are {age} years old.</p>
      <ul>
        {[1, 2, 3].map(n => <li key={n}>{n}</li>)}
      </ul>
    </div>
  );
}

function App() {
  return (
    <>
      <Greeting name="Alice" age={30} />
      <Greeting name="Bob" age={25} />
    </>
  );
}
