use std::process;

use clap::{Parser, command};
use why_lib::Module;

#[derive(Parser, Debug, serde::Serialize, serde::Deserialize)]
#[command(author, version, about)]
#[command(propagate_version = true)]
pub struct VCArgs {
    /// The path to the source file.
    #[arg(index = 1)]
    pub file: std::path::PathBuf,

    /// Print the lexed source tree.
    #[arg(short = 'l', long)]
    pub print_lexed: bool,

    /// Print the parsed AST.
    #[arg(short = 'p', long)]
    pub print_parsed: bool,

    /// Print the typechecked AST.
    #[arg(short = 'c', long)]
    pub print_checked: bool,

    /// Print the validated AST.
    #[arg(short = 'v', long)]
    pub print_validated: bool,

    #[arg(short, long, default_value = "a.out")]
    pub output: std::path::PathBuf,
}

impl VCArgs {
    pub fn init() -> Self {
        VCArgs::parse()
    }
}

pub fn compile_file(args: VCArgs) -> anyhow::Result<()> {
    let module = Module::new(args.file.to_str().map(|path| path.to_string()).expect(""));

    let module = module.lex()?;

    if args.print_lexed {
        println!("{:#?}", module.inner);
    }

    let module = match module.parse() {
        Ok(module) => module,
        Err(e) => {
            eprintln!("{e}");
            process::exit(-1);
        }
    };

    if args.print_parsed {
        println!("{:#?}", module.inner);
    }

    let module = match module.check() {
        Ok(module) => module,
        Err(e) => {
            eprintln!("{e}");
            process::exit(-1);
        }
    };

    if args.print_checked {
        println!("{:#?}", module.inner);
    }

    let module = match module.validate() {
        Ok(module) => module,
        Err(e) => {
            eprintln!("{e}");
            process::exit(-1);
        }
    };

    if args.print_validated {
        println!("{module:#?}");
    }

    module.codegen(args.output.to_str().unwrap());

    Ok(())
}
