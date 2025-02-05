fn main() {
    let compile_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let state_dir = std::env::var("OUT_DIR").unwrap();
    let cargo_bin_name = std::env::var("CARGO_BIN_NAME").unwrap_or_else(|_| "geekorm".to_string());

    let state_file = format!("geekorm-{}-{}.json", cargo_bin_name, compile_time);

    println!(
        "cargo:rustc-env=GEEKORM_STATE_FILE={}/{}",
        state_dir, state_file
    );
}
