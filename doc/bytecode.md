
# Bytecode

This is general outline of how the bytecode is supposed to work.

## TODO

Change the way functions and calls work to make it possible to
call any closure in the value stack.

- The definition of `Value` will need to change so `Closure` points
to the `instructions` array instead of containing a reference to
the AST.

- Probably move `env.rs` and `value.rs` to `exec/`, and also part
of `errors.rs`, and create their bytecode counterparts in
`bytecode/`. [QUESTION: how to handle both error types -- `exec/`'s
and `bytecode/`'s in `lib.rs`? Using trait objects will probably
slow down AST execution of `return` and `break` a lot. Or maybe not...?]


## Bytecode Program

This is the result of the bytecode compilation (but `global_functions` will
probably be replaced by the new global environment, see the TODO):

- `instructions: &[u32]`

    - Instructions array of size up to `2^26`.

- `global_functions: HashMap<String, u32>`

    - Map from function names to instruction pointers. Each function points to
      a location in the instruction array that contains the number of parameters
      expected by the function followed by the list of instructions in the function.
      The last instruction of the function should be a `ret`, which should be the
      last element of the array or be followed by another function (that is, the
      number of parameters expected by the next function, etc.).

## Execution state

- `IP : usize`

    - Offset of the current instruction in `instructions`.

- `cur_env : Rc<Env>`

    - Current environment.

- `val_stack : Vec<Value>`

    - Value stack.

- `ret_stack : Vec<usize>`

    - Return stack.

- `flag_true : bool`

    - Result of the last `test` opcode.


## Execution

The user asks to call a given global function with a given array of arguments.

1. Get the instruction pointer `IP` from the `global_functions` map.

2. Read the number of parameters expected by the function from `instructions[IP]`, and
   check if it matches the number of arguments given.

3. Create a new environment with the given arguments, and the parent set to the global
   environment. Set `cur_env` to this new environment.

4. Empty the stacks and zero all the flags.

5. Increment `IP` (to skip the number of parameters of the function) and start executing
   instructions.

## Opcodes

Each instruction is an element of the `instructions` array (`u32`).  We use the notation
`[A..B]` to refer to all bits from `A` (high) to `B` (low) of an instruction, so for
example `[1..0]` refers to the lowest two bits.

The opcode of an instruction is at bits `[31..26]` (that is, the 6 highest bits).
This section describes some of the opcodes, the location of the bits of the targets (if
present), and some pseudocode that describes what the instruction is supposed to do.


### newenv

`newenv N`

- `N` at `[11..0]`

Pops `N` values from the value stack and creates a new environment with them.

Execution:

```
[tmp] = for i in 1..N { val_stack.pop() }
cur_env = make_new_env(parent = cur_env, [tmp])
```
    

### popenv

`popenv`

Discards the current environment, returning to its parent.

Execution:

```
cur_env = cur_env.parent
```

Errors if `cur_env` is the global environment.


### getvar

`getvar VI, EI`

- `VI` at `[23..12]`
- `EI` at `[11..0]`

Reads a variable from the environment.

Execution:

```
tmp = cur_env(VI, EI)
val_stack.push(tmp)
```

Errors if `(VI, EI)` is not in the environment.


### setvar

`setvar VI, EI`

- `VI` at `[23..12]`
- `EI` at `[11..0]`

Writes a value to the environment.

Execution:

```
tmp = val_stack.pop()
cur_env(VI, EI) = tmp
```

Errors if `(VI, EI)` is not in the environment.


### test

`test`

Pops a value from the value stack and test its truth value.

Execution:

```
tmp = stack.pop()
flag_true = tmp.is_true()
```


### jmp

`jmp T`

- `T` at `[25..0]`

Jumps to another instruction.

Execution:

```
IP = T
```


### jmp_true

`jmp_true T`

- `T` at `[25..0]`

Jumps to another instruction if the "true flag" is set.

Execution:

```
if flag_true then IP = T
```


### newenv_func

`newenv_func T`

- `T` at `[25..0]`

Creates a new environment for a function call. Used in conjunction with `call`.

Execution:

```
[tmp] = for i in 1..instructions[T] { stack.pop() }
cur_env = make_new_env(parent = cur_env, local vars = [tmp])
```

### call

`call T`

- `T` at `[25..0]`

Calls a function

Execution:

```
ret_stack.push(IP)
IP = T+1
```


### ret

`ret`

Returns from a function.

Execution:

```
IP = ret_stack.pop()
```
