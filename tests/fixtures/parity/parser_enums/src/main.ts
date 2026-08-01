// Enum declarations
enum Direction {
  Up = 0,
  Down = 1,
  Left = 2,
  Right = 3,
}

enum Color {
  Red = "RED",
  Green = "GREEN",
  Blue = "BLUE",
}

const enum NumberEnum {
  One = 1,
  Two = 2,
  Three = 3,
}

// Enum with computed values
enum E {
  A = 1 << 0,
  B = 1 << 1,
  C = A | B,
}

// Using enums
function move(dir: Direction): void {
  switch (dir) {
    case Direction.Up: break;
    case Direction.Down: break;
    default: break;
  }
}
