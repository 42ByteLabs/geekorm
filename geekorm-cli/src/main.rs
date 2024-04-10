use anyhow::Result;

fn main() -> Result<()> {
    println!(
        "{}  - v{}\n",
        geekorm::GEEKORM_BANNER,
        geekorm::GEEKORM_VERSION
    );
    Ok(())
}
