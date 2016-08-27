## BleepScript

A simple toy scriting language written in Rust, inspired by SICP's
[metacircular evaluator](https://mitpress.mit.edu/sicp/full-text/sicp/book/node76.html).

## Loading and Executing a Script

```Rust
let mut bleep = Bleep::new();

bleep.load_script("script_file.bs").unwrap();

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

function make_counter(start) {
    return function() {
        return start = start + 1;
    };
}

function test() {
    var c1 = make_counter(0);
    var c2 = make_counter(10);
    printf("%d, %d\n", c1(), c2());    # prints 1, 11
    printf("%d, %d\n", c1(), c2());    # prints 2, 12

    if (c1() == 3) {
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

## Design Limitations

Values refer to other values and the environment via reference counting
(using Rust's [Rc&lt;T&gt;](https://doc.rust-lang.org/std/rc/struct.Rc.html)).
Because of this, any loops in data structures (including closure references)
will cause memory leaks. This may be fixed if/when Rust adds [suport for Garbage
Collection](http://manishearth.github.io/blog/2016/08/18/gc-support-in-rust-api-design/).

## License

MIT License ([License.txt](https://github.com/ricardo-massaro/bleepscript/blob/master/License.txt))
