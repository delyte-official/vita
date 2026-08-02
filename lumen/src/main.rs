use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("usage: lumen <file.vt>");
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);

    let source = match std::fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading {}: {e}", input_path.display());
            std::process::exit(1);
        }
    };

    let output_path = input_path.with_extension("");

    match lumen::compile(&source, &output_path) {
        Ok(()) => {
            println!("compiled: {}", output_path.display());
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
