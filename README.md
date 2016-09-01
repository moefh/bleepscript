## BleepScript

A toy script language written in Rust, inspired by SICP's
[metacircular evaluator](https://mitpress.mit.edu/sicp/full-text/sicp/book/node76.html).

Run the examples:

```text
cargo run scripts/main.tst "Hello, world!"

cargo run scripts/draw_mandelbrot.tst
```

## Loading and Executing a Script from Rust

```Rust
extern crate bleepscript;

use bleepscript::Bleep;

let mut bleep = Bleep::new();

bleep.load_file("script_file.bs").unwrap();

let ret = bleep.call_function("main", &[]).unwrap();
println!("script returned {}", ret);
```

## Example Script

```javascript
function main() {
    printf("Hello, world!\n");
    test();
    return 0;
}

function make_counter(num) {
    return {
        next : function() {
            num = num + 1;
        },

        read : function() {
            return num;
        },
    };
}

function test() {
    var c1 = make_counter(0);
    var c2 = make_counter(10);
    c1.next();
    c2.next();
    printf("%d, %d\n", c1.read(), c2.read());    # prints 1, 11

    c1.next();
    if (c1.read() == 2) {
        printf("ok!\n");
    } else {
        error("this will not happen");
    }

    var i = 1;
    while (i <= 10) {
        if (i == 6)
            break;
        printf("%d\n", i);
        i = i + 1;
    }
    
    return;
    error("this will not happen");
}
```

## Bytecode Compiler/Interpreter

Running from the AST works, but there's also an experimental bytecode compiler and interpreter.
It currently doesn't support maps or vectors, and is slower than AST execution.

To use see it in action, run

`cargo run -- --bytecode scripts/draw_mandelbrot.tst`

To use it from Rust, just use `Bleep::compile_file()` instead of `Bleep::load_file()`. See the code
at `src/bin/main.rs` for more details.


## Design Limitations

Values refer to other values and the environment via reference counting
(using Rust's [Rc&lt;T&gt;](https://doc.rust-lang.org/std/rc/struct.Rc.html)).
Because of this, any loops in data structures (including closure references)
will cause memory leaks. This may be fixed if/when Rust adds [suport for Garbage
Collection](http://manishearth.github.io/blog/2016/08/18/gc-support-in-rust-api-design/).

## License

MIT License ([License.txt](https://github.com/ricardo-massaro/bleepscript/blob/master/License.txt))
