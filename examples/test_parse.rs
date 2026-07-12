use tsox::parser::Parser;

fn main() {
    let sources = [
        ("test1.ts", r#"
// Decorators
function log(target: any, key: string, desc: PropertyDescriptor) {}
class MyService {
  @log
  method() {}
}

// Generics with constraints
function identity<T extends string>(x: T): T { return x; }

// Conditional types
type IsString<T> = T extends string ? true : false;

// Mapped types
type Readonly2<T> = { readonly [K in keyof T]: T[K]; };

// Template literal types
type Hello = `hello ${string}`;

// Enum
enum Color { Red = "red", Green = "green", Blue = "blue" }

// const enum
const enum Direction { Up, Down, Left, Right }

// Namespace
namespace App {
  export function init() {}
  export interface Config {}
}

// Module declaration
declare module "express" {
  export interface Request {}
}

// Abstract class
abstract class Animal {
  abstract makeSound(): void;
  protected name: string;
  constructor(name: string) { this.name = name; }
}

// Type guards
function isFish(pet: any): pet is Fish {
  return (pet as any).swim !== undefined;
}

// never type
function fail(msg: string): never {
  throw new Error(msg);
}

// infer
type GetReturn<T> = T extends (...args: any[]) => infer R ? R : never;

// Satisfaction
const config = { a: 1 } satisfies Record<string, number>;
"#),
        ("test3.ts", r#"
// Async/await
async function fetchData(url: string): Promise<Response> {
  const response = await fetch(url);
  return response;
}

// Optional chaining and nullish coalescing
const value = obj?.foo?.bar ?? "default";
const length = str?.length;

// Destructuring
const { a, b: renamed, ...rest } = obj;
const [first, , third, ...rest2] = arr;

// Spread
const merged = { ...obj1, ...obj2 };
const arr2 = [...arr1, ...arr2];

// Generators
function* counter() {
  let i = 0;
  while (true) {
    yield i++;
  }
}

// Async generators
async function* asyncGen() {
  yield await fetch("url");
}

// for await
async function process(stream: ReadableStream) {
  for await (const chunk of stream) {
    console.log(chunk);
  }
}

// Labeled tuple types
type Point = [x: number, y: number];

// Template literal types
type EventName = `on${Capitalize<string>}`;

// Key remapping
type Getter<T> = {
  [K in keyof T as `get${Capitalize<string & K>}`]: () => T[K];
};

// Optional variance annotations
interface Box<out T> {
  value: T;
}
"#),
    ];

    for (name, source) in &sources {
        let (file, diagnostics) =
            Parser::parse_source_file_text_with_diagnostics(name, source.to_string());
        println!("{}: {} diagnostics", name, diagnostics.len());
        for d in &diagnostics {
            let pos = d.range.pos as usize;
            let end = d.range.end as usize;
            let (line, col) = tsox::diagnosticwriter::line_and_character(
                &file.line_map,
                &file.text,
                pos,
            );
            println!(
                "  [{}:{}] {} args={:?}",
                line + 1,
                col + 1,
                d.message.text,
                d.message_args
            );
            let snippet_end = (end).min(pos + 40).min(file.text.len());
            if pos < file.text.len() {
                let snippet = &file.text[pos..snippet_end];
                println!("    snippet: {:?}", snippet);
            }
        }
    }
}
