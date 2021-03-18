# 言語

GenGo (言語), est un langage de programmation implémenté dans le cadre du cours _informatique fondamentale & compilation_ des Mines.

```shell
# Compile & run a program
cargo run -- comp \
    --file [filename] \  # Filename with Gengo code
    --ir [filename]      # Optionnal param to save LLVM IR to a file

# JIT
cargo run -- jit
```

## Syntaxe d'un programme

```shell
/*
Commentaires
*/

# Variables
let a = 3;
let b = 3;
a = b = 1;

# Function (can be recursive)
fn (a, b) {
    let a = 5;
    return 5;
}

# If-Else condition
if cond
then {
    ... ;
    ... ;
} else {
    ... ;
    ... ;
}

# While loop
while cond {
    ... ;
}

# Global variables
global a = 3;
fn test () {
    a
}

5 # Retour implicite sans ;
```

## Tests

```rust
# Run the tests
cargo test -- --test-threads=1 --nocapture
```

## Improvements

- [ ] Draw a clear distinction between expressions (binaryop, unaryop, etc.) & items (assignement, loops, etc.)
- [ ] Add structs & arrays
