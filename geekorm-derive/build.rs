fn main() {
    let compile_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let state_dir = std::env::var("OUT_DIR").unwrap();
    let state_file = format!("geekorm-{}.json", compile_time);

    println!(
        "cargo:rustc-env=GEEKORM_STATE_FILE={}",
        state_dir + "/" + &state_file
    );
}
