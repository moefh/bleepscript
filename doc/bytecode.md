
# Bytecode

This is general outline of how the bytecode is supposed to work.

## TODO

(...)

## Compiled Program

This is the result of the bytecode compilation:

- `instructions: Vec<u32>`

    - Instructions array of size up to `2^26`.

## Execution state

- `ip : usize`

    - Offset of the current instruction in `instructions`.

- `env : Rc<Env>`

    - Current environment.

- `val_stack : Vec<Value>`

    - Value stack.

- `ret_stack : Vec<usize>`

    - Return stack.

- `flag_true : bool`

    - Result of the last `test` opcode.


## Execution

The user asks to call a given named function with a given array of arguments.

1. Get the value of the function name from the environment. If it's not a Value::BCClosure,
   call `Value::call()` on it, and we're done (no bytecode to execute).

2. Check that the number of arguments passed matches the closure's `.num_params`. 

3. Create a new environment with the given arguments, and with the parent set to the global
   environment. Set `env` to this new environment.

4. Empty the stacks and zero all the flags.

5. Set `IP` to the closure's `.addr` and start executing the instructions.


## Opcodes

Each instruction is an element of the `instructions` array (`u32`).  We use the notation
`[A..B]` to refer to all bits from `A` (high) to `B` (low) of an instruction, so for
example `[1..0]` refers to the lowest two bits.

The opcode of an instruction is at bits `[31..26]` (that is, the 6 highest bits).
This section describes some of the opcodes, the location of the bits of the targets (if
present), and some pseudocode that describes what the instruction is supposed to do.


### newenv

`newenv N T`

- `N` at `[23..12]`
- `T` at `[11..0]`

Pops `N` values from the value stack and creates a new environment with `T` values.
The first `N` values will be the ones popped from the stack, the rest will be
`Value::Null`.

Execution:

```
[tmp] = for i in 1..N { val_stack.pop() }
env_stack.push(env);
env = make_new_env(parent = env, [tmp, ...])
```
    

### popenv

`popenv N`

- `N` at `[11..0]`

Pops N environments from the env stack.

Execution:

```
for _ in 0..N {
  env = env_stack.pop()
}
```

Errors if the global environment is reached before the end.


### getvar

`getvar VI, EI`

- `VI` at `[23..12]`
- `EI` at `[11..0]`

Reads a variable from the environment.

Execution:

```
tmp = env(VI, EI)
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
env(VI, EI) = tmp
```

Errors if `(VI, EI)` is not in the environment.


### setelem

`setelem`

Sets an element of a map or array.

Execution:

```
val = val_stack.pop()
key = val_stack.pop()
c = val_stack.pop()
c[key] = val
val_stack.push(val)
```

Errors if `(VI, EI)` is not in the environment.


### pushval

`pushval N`

- `N` at `[25..0]`

Pushes a literal value to the value stack.

Execution:

```
val = get_literal(N)
val_stack.push(val)
```

Errors if `N` is not a literal value.


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

Calls a function passing `N` parameters.

Execution:

```
[args] = for _ in 0..N { val_stack.pop() }
func = val_stack.pop()
match func {
  native function => {
    tmp = func.call([args], env)
    val_stack.push(tmp)
  }
  AST closure => {
    check N == func.num_param
    call_env = make_new_env(func.env, [args])
    tmp = func.run_function_body(call_env)
    val_stack.push(tmp)
  }
  bytecode closure => {
    check N == func.num_param
    env_stack.push(env)
    env = make_new_env(parent = env, [args])
	ret_stack.push(IP)
	IP = func.IP
  }
  _ => ERROR
}
```

Errors if the value being called is not a function or if N is not equal to the number of parameters of the function.


### ret

`ret`

Returns from the current function.

Execution:

```
env = env_stack.pop()
IP = ret_stack.pop()
```
