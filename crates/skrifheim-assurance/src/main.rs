#![forbid(unsafe_code)]
use skrifheim_assurance::{TARGETS, seeds, smoke};
use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [mode, target] if mode == "smoke" => {
            let selected: &[&str] = if target == "all" {
                &TARGETS
            } else {
                &[target.as_str()]
            };
            for name in selected {
                println!("{name}: {} cases", smoke(name)?);
            }
        }
        [mode, target, destination] if mode == "seeds" => {
            let cases = seeds::for_target(target)?;
            let directory = PathBuf::from(destination);
            fs::create_dir_all(&directory)?;
            for (index, case) in cases.iter().enumerate() {
                fs::write(directory.join(index.to_string()), case)?;
            }
            println!("{target}: {} seeds", cases.len());
        }
        _ => {
            return Err(
                "usage: skrifheim-assurance smoke TARGET|all OR seeds TARGET DIRECTORY".into(),
            );
        }
    }
    Ok(())
}
