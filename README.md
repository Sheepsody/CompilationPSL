# 言語

GenGo (言語), is an experimental programming langage I made for the course _informatique fondamentale & compilation_ at Mines ParisTech.

## How to use ?

```shell
# Compile & run a program
cargo run -- comp \
    --file [filename] \  # Filename with Gengo code
    --ir [filename]      # Optionnal param to save LLVM IR to a file

# JIT
cargo run -- jit
```

## Syntax

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
    a = 1;
    return a; # Returns 1
}

5 # Retour implicite sans ;
```

## Running the tests

```shell
# Run the tests
cargo test -- --test-threads=1 --nocapture
```
