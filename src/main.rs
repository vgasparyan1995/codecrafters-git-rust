use anyhow::{ensure, Context, Result};
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::{
    format, fs,
    io::{BufRead, BufReader, Read, Write},
};

#[derive(Parser)]
#[command(about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    // Create an empty Git repository or reinitialize an existing one.
    Init,

    // Provide contents or details of repository objects.
    CatFile {
        // Pretty-print the contents of <object> based on its type.
        #[arg(short)]
        pretty_print: bool,

        // The name of the object to show.
        object: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init => init()?,
        Command::CatFile {
            pretty_print,
            object,
        } => cat_file(object, pretty_print)?,
    };
    Ok(())
}

fn init() -> Result<()> {
    fs::create_dir_all(".git/objects").context("Creating '.git/objects'")?;
    fs::create_dir_all(".git/refs").context("Creating '.git/refs'")?;
    fs::write(".git/HEAD", "ref: refs/heads/main\n").context("Writing HEAD")?;
    Ok(())
}

fn cat_file(object: String, pretty: bool) -> Result<()> {
    ensure!(object.len() == 40, "<object> hash must be of length 40.");
    ensure!(pretty, "how else?");
    let (dir, file) = object.split_at(2);
    let path = format!(".git/objects/{dir}/{file}");
    let file_content = fs::read(path)?;
    let mut decoder = BufReader::new(ZlibDecoder::new(&file_content[..]));
    let mut blob_header: Vec<u8> = Vec::new();
    decoder.read_until(b'\0', &mut blob_header)?;
    ensure!(blob_header.starts_with(b"blob "), "blob header prefix");
    ensure!(blob_header.ends_with(b"\0"), "blob header suffix");
    let begin = "blob ".len();
    let end = blob_header.len() - 1;
    let size = std::str::from_utf8(&blob_header[begin..end])?.parse::<usize>()?;
    let mut content = vec![b'\0'; size];
    decoder.read_exact(&mut content)?;
    std::io::stdout().write(&content)?;
    Ok(())
}
