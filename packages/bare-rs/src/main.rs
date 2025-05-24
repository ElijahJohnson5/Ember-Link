use bare_rs::{ast::Options, lexer::Lexer, parser::Parser as BareParser};
use clap::{Parser, Subcommand};
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "bare-rs", version = "0.1.0", about = "A tool for compiling bare schemas")]
#[command(next_line_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
#[command(next_line_help = true)]
enum Commands {
    /// Compiles a bare schema file to Rust code
    Compile {
        /// The path to the input schema file
        #[arg(required = true)]
        schema: String,

        /// The path to the output file
        #[arg(short, long)]
        out: String,

        /// The custom derives to add to the generated Rust code
        #[arg(short, long, value_delimiter = ',')]
        derives: Option<Vec<String>>,

        /// The custom use statements to add to the generated Rust code example: --uses "use std::collections::HashMap"
        #[arg(short, long)]
        uses: Option<Vec<String>>,

        #[arg(long)]
        serde: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Compile { schema, out, derives, uses, serde } => {
            println!("Compiling schema from {} to {}", schema, out);

            if !schema.ends_with(".bare") {
                println!("Schema must be a bare schema file (.bare)");
                return;
            }

            let contents = fs::read_to_string(schema).expect(&format!("Could not read schema file: {}", schema));
            
            let tokens = match Lexer::new(&contents).lex() {
                Ok(tokens) => tokens,
                Err(e) => {
                    println!("Error lexing schema: {}", e);
                    return;
                }
            };
            
            let ast = match BareParser::new(tokens).parse_user_types() {
                Ok(ast) => ast,
                Err(e) => {
                    println!("Error parsing schema: {}", e);
                    return;
                }
            };

            let (serde_union_string, serde_string) = if *serde {
                (Some("#[serde(tag = \"tag\", content = \"val\")]".to_string()), Some("#[serde(rename_all = \"camelCase\")]".to_string()))
            } else {
                (None, None)
            };

            let options = Options {
                serde_string: &serde_string,
                serde_string_union: &serde_union_string,
                derives: derives,
                uses: uses,  
            };

            let output = ast.to_rust(&options);
            fs::write(out, output).expect(&format!("Could not write to output file: {}", out));

            println!("Schema compiled successfully to {}", out);
        }
    }
}