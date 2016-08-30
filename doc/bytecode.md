
# Bytecode

This is general outline of how the bytecode is supposed to work.

## Compiled Program

This is the result of the bytecode compilation:

- `instructions: &[u32]`

    - Instructions array of size up to `2^26`.

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

The user asks to call a given named function with a given array of arguments.

1. Get the value of the function name from the environment, and call its method `call()`.

2. `Value::call()` for a bytecode-compiled function (type `Value::BytecodeClosure`
   [TODO: change name??]) checks the number of arguments passed 
    the function's `IP` and the number of 

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


### call

`call N`

- `N` at `[11..0]`

Calls a function passing `N` parameters.  This instruction must be preceded by `newenv N`.

Execution:

```
func = val_stack.pop()
match func {
  native function => {
    tmp = func.call([args], cur_env)
    val_stack.push(tmp)
  }
  AST closure => {
    check N == func.num_param
    tmp = func.run_function_body(cur_env)
    val_stack.push(tmp)
  }
  bytecode closure => {
    check N == func.num_param
	ret_stack.push(IP)
	IP = func.IP
  }
  _ => ERROR
}
```

Errors if the value being called is not a function or if N is not equal to the number of parameters of the function.


### ret

`ret`

Returns from a function.  This instruction must be preceded by `popenv`.

Execution:

```
IP = ret_stack.pop()
```
