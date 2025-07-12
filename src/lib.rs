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

    /// Force compiler pipeline.
    #[arg(short = 'f', long)]
    pub force: bool,

    #[arg(short, long, default_value = "a.out")]
    pub output: std::path::PathBuf,
}

impl VCArgs {
    pub fn init() -> Self {
        VCArgs::parse()
    }
}

pub fn compile_file(args: VCArgs) -> anyhow::Result<()> {
    let module = Module::new(args.file.to_str().map(|path| path.to_string()).expect(""))?;

    if !module.exists() || args.force {
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

        module.codegen();
    } else {
        if args.print_lexed {
            eprintln!(
                "[WARN] CLI argument '-l' | '--print-lexed' ignored since module is already present! Use '-f' to run the compiler pipeline!"
            );
        }

        if args.print_parsed {
            eprintln!(
                "[WARN] CLI argument '-p' | '--print-parsed' ignored since module is already present! Use '-f' to run the compiler pipeline!"
            );
        }

        if args.print_checked {
            eprintln!(
                "[WARN] CLI argument '-c' | '--print-checked' ignored since module is already present! Use '-f' to run the compiler pipeline!"
            );
        }

        if args.print_validated {
            eprintln!(
                "[WARN] CLI argument '-v' | '--print-validated' ignored since module is already present! Use '-f' to run the compiler pipeline!"
            );
        }
    }

    module.compile(args.output.to_str().unwrap());

    Ok(())
}
