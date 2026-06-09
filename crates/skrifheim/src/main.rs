fn main() {
    let info = skrifheim::build_info();
    println!(
        "{} {} (Rust baseline {})",
        info.database_name, info.version, info.rust_baseline
    );
}
