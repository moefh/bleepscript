
# BleepScript

BleepScript is a library to parse and run scripts written in a toy language.

## Directory `src/readers`

Reading scripts is handled by objects implementing the `InputSource` trait.
The `InputSource::open()` method opens a named input (e.g., a file) and
returns an iterator of `Result<char,ReadError>`. Two objects implement this
trait: `FileOpener` and `StringInputOpener`, used internally by the
`Bleep::load_file()` and `Bleep::load_string()`. The `Bleep::load_user()`
can be used to load a script from any object implementing the `InputSource`
trait.
 
## Directory `src/parser`
 
The `Parser` struct implements a recursive descent parser that reads
function definitions from the input. It uses a `Tokenizer` to read tokens
using the iterators returned by `InputSource::open()`, and builds an
abstract syntax tree with objects defined in the `ast` directory.

## Directory `src/ast`

Each function definition is parsed into an AST whose root is a
`ast::NamedFuncDef` (defined in `ast/statements.rs`).

AST nodes are divided into two main groups: expressions (`ast/expressions.rs`)
and statements (`ast/statements.rs`). Statements contain expressions (unary
and binary operations, function calls, etc.) but expressions can also contain
statements (in the body of an unnamed function).

Each AST node can be *analyzed* (see the method `analyze()` of each `struct`
and `enum` defined in `ast/expressions.rs` and `ast/statements.rs`) to produce
an executable form of the AST implemented in the `exec` directory.

The AST nodes can also be *compiled* (see the method `compile()` of each
`struct` and `enum` defined in `ast/expressions.rs` and `ast/statements.rs`)
to produce bytecode instructions.

## Directory `src/exec`

The `struct`s and `enum`s here are almost a mirror of the ones in the `ast`
directory, but the nodes here are optimized for execution.

Each node in the tree has an `eval()` method that evaluates the node in a
given environment. The environment is passed as a reference-counted object,
because some nodes produce results that need to keep a reference to the
environment -- for example, when the node for an unnamed function definition
is evaluated, it produces a `Closure` value that references the given
environment.

## Directory `src/bytecode`

The bytecode generator (in `bytecode/gen.rs`) is used by the AST nodes to
produce bytecode instructions.

 The bytecode runner (in `bytecode/run.rs`) is used by the `Bleep` object to
 execute a bytecode-compiled function definition.

## Directory `src/bin`

This directory contains example programs using the library and benchmarks. 

## Directory `src` itself

### `Bleep`

The `struct Bleep` (in `lib.rs`) stores the current state of the
script, and has methods for:

 - adding/changing values in the global environment (an `Env` struct)
 
 - reading script files and adding the functions to the global environment
   (either in the executable-AST form or in bytecode form)

 - running functions (both executable-AST and bytecode)

### `Env`

The `struct Env` (in `env.rs`) stores the values of an environment
and optionally references its parent environment (if it's not the global
environment).

Every use of this struct through the library is reference-counted, since
values in an environment can contain references to other environments
(then closures capture their environment).

### `Value`

The `struct Value` (in `value.rs`) represents a value of the script language.
`Value`s are stored in the environments and passed to and returned from
script functions.

### Native Functions

The script functions that are implemented in Rust (like `printf` and all
the operators) are defined in `native.rs`.  A native function receives an
array of `Value`s containing the arguments and a reference to the environment
where the function is being executed on. The environment is usually ignored,
but can be used to implement functions for debugging or other more things that
are not possible to express in the script language itself (like changing the
environment in arbitrary ways, accessing variables that are shadowed by other
variables, etc).
