use clap::{Parser, Subcommand};
use console::style;
use geekorm::{GEEKORM_BANNER, GEEKORM_VERSION};
use std::path::PathBuf;

pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Arguments {
    /// Enable Debugging
    #[clap(long, env, default_value_t = false)]
    pub debug: bool,

    /// Disable Banner
    #[clap(long, default_value_t = false)]
    pub disable_banner: bool,

    /// Configuration file path
    #[clap(short, long, env, default_value = "./.geekorm.yml")]
    pub config: PathBuf,

    /// Working Directory
    #[clap(short, long, env, default_value = "./")]
    pub working_dir: PathBuf,

    /// GeekORM Path (mainly for development)
    #[clap(long, env = "GEEKORM_PATH")]
    pub geekorm_path: Option<PathBuf>,

    /// Subcommands
    #[clap(subcommand)]
    pub commands: Option<ArgumentCommands>,
}

#[derive(Subcommand, Debug)]
pub enum ArgumentCommands {
    /// Initialize GeekORM
    Init,
    /// Migration commands
    Migrate {
        #[clap(short, long, default_value_t = false)]
        data: bool,
    },
    /// Update
    Update,
    /// Test the migrations
    Test,
    /// Read and display the database schema generated by GeekORM
    Display,
}

pub fn init() -> Arguments {
    let arguments = Arguments::parse();

    let log_level = match &arguments.debug {
        false => log::LevelFilter::Info,
        true => log::LevelFilter::Debug,
    };

    env_logger::builder()
        .parse_default_env()
        .filter_level(log_level)
        .format(|buf, record| {
            use std::io::Write;

            let color = match record.level() {
                log::Level::Error => console::Color::Red,
                log::Level::Warn => console::Color::Yellow,
                log::Level::Info => console::Color::Green,
                log::Level::Debug => console::Color::Blue,
                log::Level::Trace => console::Color::Cyan,
            };

            writeln!(
                buf,
                "[{:<5}] {}",
                style(record.level()).fg(color),
                record.args()
            )
        })
        .init();

    if !arguments.disable_banner {
        println!(
            "{}    {} - v{}\n",
            style(GEEKORM_BANNER).green(),
            style(AUTHOR).red(),
            style(GEEKORM_VERSION).blue()
        );
    }

    arguments
}
