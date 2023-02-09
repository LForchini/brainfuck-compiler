// Compilation steps
// 1. Read file
// 2. Lex file
// 3. Generate intermediate code
// 4. Perform optimisations (++ ++ => +=2)
// 5. Generate nasm(?) assembly
// 6. Assembly generated code
mod lex;
mod profile;

use clap::Parser;
use lex::Token;
use profile::Profile;
use std::{fs, path::Path};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about=None)]
struct Args {
    /// Filename of the brainfuck program
    infile: String,

    /// Name of the output file
    #[arg(short = 'o', long = "out")]
    outfile: Option<String>,

    /// Output an assembly file
    #[arg(short = 'a', long = "asm")]
    output_assembly: bool,

    /// Select which profile to assemble with
    #[arg(short = 'p', long = "profile")]
    profile: Option<String>,
}

fn gen_file_names(args: &Args) -> (String, String, String) {
    let infile = args.infile.clone();
    let base = infile.split('.').collect::<Vec<&str>>()[0];

    let asmfile = if args.output_assembly {
        if let Some(outfile) = &args.outfile {
            outfile.clone()
        } else {
            format!("{base}.s")
        }
    } else {
        format!("{base}.s")
    };

    let outfile = if let Some(outfile) = &args.outfile {
        outfile.clone()
    } else {
        base.to_string()
    };

    (infile, asmfile, outfile)
}

fn read_bf_file(filename: &String) -> String {
    fs::read_to_string(filename).expect("Could not read file")
}

fn generate_asm(profile: &Profile, tokens: Vec<Token>) -> Vec<String> {
    let mut lines = vec![profile.get_setup_asm()];
    for tok in tokens {
        lines.push(profile.get_asm(tok));
    }
    lines.push(profile.get_teardown_asm());

    lines
}

fn main() {
    pretty_env_logger::init();
    log::info!("Enabled logging");

    let args = Args::parse();
    log::info!("Read args: {:?}", args);

    let (infile, asmfile, execfile) = gen_file_names(&args);

    let file_contents = read_bf_file(&infile);
    log::debug!(
        "Read file: {:#?} ({:#?} chars)",
        &args.infile,
        file_contents.len()
    );

    let tokens = lex::lex(&file_contents);
    log::debug!("Lexed to {:#?} symbols", tokens.len());

    let optimised_tokens = lex::optimise_tokens(tokens);
    log::debug!("Optimised to {:#?} symbols", optimised_tokens.len());

    let profile = if let Some(profile_name) = &args.profile {
        Profile::get_by_string(profile_name).expect("Profile not found")
    } else {
        Profile::default()
    };
    log::trace!("Using profile: {:#?}", profile);

    let asm = generate_asm(profile, optimised_tokens);
    log::debug!("Generated assembly");

    if args.output_assembly {
        Profile::write_asm(&asm, Path::new(&asmfile)).unwrap();
    } else {
        profile.generate_bin(&asm, Path::new(&execfile)).unwrap();
    }
}
