// Compilation steps
// 1. Read file
// 2. Lex file
// 3. Generate intermediate code
// 4. Perform optimisations (++ ++ => +=2)
// 5. Generate nasm(?) assembly
// 6. Assembly generated code

use clap::Parser;
use std::fs;
use std::io::Write;
use std::process::Command;

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
}

fn gen_file_names(args: &Args) -> (String, String, String, String) {
    let infile = args.infile.clone();
    let base = infile.split('.').collect::<Vec<&str>>()[0];

    let asmfile = if args.output_assembly {
        if let Some(outfile) = &args.outfile {
            outfile.clone()
        } else {
            format!("{}.asm", base)
        }
    } else {
        format!("{}.asm", base)
    };

    let objfile = format!("{}.o", base);

    let outfile = if let Some(outfile) = &args.outfile {
        outfile.clone()
    } else {
        base.to_string()
    };

    (infile, asmfile, objfile, outfile)
}

fn read_bf_file(filename: &String) -> String {
    fs::read_to_string(filename).expect("Could not read file")
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Token {
    PtrAdd(usize),
    PtrSub(usize),
    Add(usize),
    Sub(usize),
    LoopStart(usize),
    LoopEnd(usize),
    PutChar,
    GetChar,
}

fn lex(contents: String) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut loop_counter = 0;
    let mut active_loops = Vec::new();

    for c in contents.chars() {
        match c {
            '>' => tokens.push(Token::PtrAdd(1)),
            '<' => tokens.push(Token::PtrSub(1)),
            '+' => tokens.push(Token::Add(1)),
            '-' => tokens.push(Token::Sub(1)),
            '[' => {
                tokens.push(Token::LoopStart(loop_counter));
                active_loops.push(loop_counter);
                loop_counter += 1;
            }
            ']' => {
                let t = active_loops.pop().expect("Unmapped loop end");
                tokens.push(Token::LoopEnd(t));
            }
            '.' => tokens.push(Token::PutChar),
            ',' => tokens.push(Token::GetChar),
            _ => {}
        }
    }

    if !active_loops.is_empty() {
        panic!("Unmatched loop start")
    }

    tokens
}

fn group_tokens(tokens: Vec<Token>) -> Vec<Token> {
    let mut new_tokens = Vec::new();

    let mut accumulator = None;
    for token in tokens {
        accumulator = match (token, accumulator) {
            (Token::PtrAdd(a), Some(Token::PtrAdd(b))) => Some(Token::PtrAdd(a + b)),
            (Token::PtrSub(a), Some(Token::PtrSub(b))) => Some(Token::PtrSub(a + b)),
            (Token::Add(a), Some(Token::Add(b))) => Some(Token::Add(a + b)),
            (Token::Sub(a), Some(Token::Sub(b))) => Some(Token::Sub(a + b)),

            (tok, Some(acc)) => {
                new_tokens.push(acc);
                Some(tok)
            }
            (tok, None) => Some(tok),
        };
    }

    if let Some(acc) = accumulator {
        new_tokens.push(acc);
    }

    new_tokens
}

fn optimise_tokens(tokens: Vec<Token>) -> Vec<Token> {
    let tokens = group_tokens(tokens);
    // TODO: Perform more optimisations

    #[allow(clippy::let_and_return)]
    tokens
}

fn generate_asm(tokens: Vec<Token>) -> Vec<String> {
    let mut lines = vec![
        "SECTION .bss".to_owned(),
        "\tbuf_start: resb 40000000 ; allocate buffer".to_owned(),
        "SECTION .text".to_owned(),
        "global _start".to_owned(),
        "_start:".to_owned(),
        "\tmov edi, buf_start ; end of boilerplate".to_owned(),
    ];

    for token in tokens {
        match token {
            Token::PtrAdd(n) => lines.push(format!("\tadd edi, {}", n)),
            Token::PtrSub(n) => lines.push(format!("\tsub edi, {}", n)),
            Token::Add(n) => lines.push(format!("\tadd byte [edi], {}", n)),
            Token::Sub(n) => lines.push(format!("\tsub byte [edi], {}", n)),

            // TODO: Check the assembly generated is correct
            Token::LoopStart(id) => lines.append(&mut vec![
                "\tcmp byte [edi], 0".to_owned(),
                format!("\tjz lbl_e_{}", id),
                format!("lbl_s_{}:", id),
            ]),
            Token::LoopEnd(id) => lines.append(&mut vec![
                "\tcmp byte [edi], 0".to_owned(),
                format!("\tjnz lbl_s_{}", id),
                format!("lbl_e_{}:", id),
            ]),

            Token::PutChar => lines.append(&mut vec![
                "\tmov eax, 0".to_owned(),
                "\tmov al, [edi]".to_owned(),
                "\tpush eax".to_owned(),
                "\tmov eax, 4".to_owned(),
                "\tmov ebx, 1".to_owned(),
                "\tmov ecx, esp".to_owned(),
                "\tmov edx, 1".to_owned(),
                "\tint 80h".to_owned(),
            ]),
            Token::GetChar => lines.append(&mut vec![
                "\tmov edx, 1".to_owned(),
                "\tmov ecx, edi".to_owned(),
                "\tmov ebx, 0".to_owned(),
                "\tmov eax, 3".to_owned(),
                "\tint 80h".to_owned(),
            ]),
        };
    }

    lines.append(&mut vec![
        "\tmov ebx, 0 ; return status 0".to_owned(),
        "\tmov eax, 1 ; invoke SYS_EXIT".to_owned(),
        "\tint 80h".to_owned(),
    ]);

    lines
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .env()
        .init()
        .unwrap();
    log::info!("Enabled logging");

    let args = Args::parse();
    log::info!("Read args: {:#?}", args);

    let (infile, asmfile, objfile, execfile) = gen_file_names(&args);

    let file_contents = read_bf_file(&infile);
    log::debug!(
        "Read file: {:#?} ({:#?} chars)",
        &args.infile,
        file_contents.len()
    );

    let tokens = lex(file_contents);
    log::debug!("Lexed to {:#?} symbols", tokens.len());
    log::trace!("Tokens: {:?}", tokens);

    let optimised_tokens = optimise_tokens(tokens);
    log::debug!("Optimised to {:#?} symbols", optimised_tokens.len());
    log::trace!("Optimised tokens: {:?}", optimised_tokens);

    let asm = generate_asm(optimised_tokens);
    log::debug!("Generated {:#?} lines of assembly", asm.len());
    log::trace!("Assembly: {:#?}", asm);

    let mut file = fs::File::create(&asmfile).unwrap();
    write!(file, "{}", asm.join("\n")).unwrap();

    if args.output_assembly {
        return;
    }

    let _ = Command::new("nasm")
        .args(["-f", "elf", "-o", &objfile, &asmfile])
        .output()
        .expect("Failed to assemble");

    let _ = Command::new("ld")
        .args(["-m", "elf_i386", "-o", &execfile, &objfile])
        .output()
        .expect("Failed to link");

    fs::remove_file(asmfile).unwrap();
    fs::remove_file(objfile).unwrap();
}
