#![feature(test)]

extern crate test;
extern crate bleepscript;

use std::rc::Rc;
use std::env;
use bleepscript::*;

struct Cmdline {
    use_ast : bool,
    dump : bool,
    script_filename : String,
    script_args : Vec<String>,
}

impl Cmdline {
    fn parse(mut args : std::env::Args) -> Cmdline {
        let mut use_ast = true;
        let mut dump = false;

        args.next();  // skip program filename
        
        while let Some(arg) = args.next() {
            if arg == "--help" || arg == "-h" {
                Cmdline::print_help();
                std::process::exit(0);
            }
            
            if arg == "--dump" {
                dump = true;
                continue;
            }
            
            if arg == "--ast" {
                use_ast = true;
                continue;
            }
            
            if arg == "--bytecode" {
                use_ast = false;
                continue;
            }
            
            if arg.starts_with("--") {
                println!("Unknown option: {}", arg);
                std::process::exit(1);
            }
            
            let script_filename = arg;
            let script_args = args.collect();
            return Cmdline {
                script_filename : script_filename,
                script_args : script_args,
                use_ast : use_ast,
                dump : dump,
            };
        }
        
        Cmdline::print_help();
        std::process::exit(0);
    }
    
    fn print_help() {
        println!("USAGE: bleep [--ast|--bytecode] [options] SCRIPT_FILENAME [SCRIPT_ARGS ...]");
        println!("");
        println!("options:");
        println!(" --ast             run AST (default)");
        println!(" --bytecode        run bytecode");
        println!(" --dump            dump loaded code");
        println!("");
    }
}

fn test_function(args : &[Value], _env : &Rc<Env>) -> Result<Value,RunError> {
    match args.get(0) {
        Some(v) => println!("test_function() called from script with argument '{}'", v),
        None => println!("test_function() called from script with no arguments"),
    }
    Ok(Value::Null)
}

fn main() {
    let cmdline = Cmdline::parse(env::args());
    
    let mut bleep = Bleep::new();
    bleep.set_var("test_function", Value::new_native_func(test_function));

    if let Err(e) = {
        if cmdline.use_ast { bleep.load_file(cmdline.script_filename) }
        else { bleep.compile_file(cmdline.script_filename) }
    } {
        println!("{}", e);
        return;
    }

    //bleep.dump_env();
    if cmdline.dump {
        if cmdline.use_ast {
            bleep.dump_funcs();
        } else {
            bleep.dump_bytecode();
        }
    }
    
    //let start = time::precise_time_ns();

    let args = cmdline.script_args.iter().map(|a| Value::new_string(a)).collect::<Vec<Value>>();
    match bleep.call_function("main", &args) {
        Ok(v) => println!("-> {}", v),
        Err(e) => println!("{}", e),
    }
    
    //let end = time::precise_time_ns();
    //println!("time: {}ms", (end - start) / 1_000_000);
}

#[cfg(test)]
mod bench {
    use bleepscript::*;
    use test::Bencher;
    
    #[bench]
    fn a_bytecode(b: &mut Bencher) {
        b.iter(|| {
            let mut bleep = Bleep::new();
            bleep.compile_file("scripts/bench.tst").expect("Error loading script");
            bleep.call_function("main", &[]).expect("error running function");
        });
    }

    #[bench]
    fn b_ast(b: &mut Bencher) {
        b.iter(|| {
            let mut bleep = Bleep::new();
            bleep.load_file("scripts/bench.tst").expect("Error loading script");
            bleep.call_function("main", &[]).expect("error running function");
        });
    }
    
}
