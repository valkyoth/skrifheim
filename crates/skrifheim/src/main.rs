fn main() {
    let info = skrifheim::build_info();
    println!("{} {}", info.database_name, info.version);
}
