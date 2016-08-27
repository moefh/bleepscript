## BleepScript

A simple toy scriting language written in Rust, inspired by SICP's
[metacircular evaluator](https://mitpress.mit.edu/sicp/full-text/sicp/book/node76.html).

The code is in an early stage, it only executes a subset of the final language
(in particular, `while`, `break` and `return` are not implemented yet).

##Example Code

    function main(args) {
        print("Hello, world!\n");
        test();
    }

    function test() {
        var f = function(x) {
            print("Hello, ", x, " from anonymous function \n");
        };
        f("world");
    }

## Design Limitations

Values refer to other values and the environment via reference counting
(using Rust's [Rc&lt;T&gt;](https://doc.rust-lang.org/std/rc/struct.Rc.html)).
Because of this, any loops in data structures (including closure references)
will cause memory leaks. This may be fixed if/when Rust adds [suport for Garbage
Collection](http://manishearth.github.io/blog/2016/08/18/gc-support-in-rust-api-design/).

## License

MIT License ([License.txt](https://github.com/ricardo-massaro/bleepscript/blob/master/License.txt))
