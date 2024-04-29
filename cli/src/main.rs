use std::{env};
use std::fs::{File, remove_file};
use std::io::{Cursor, Write};
use clap::{Args, Parser, Subcommand};
use serde_derive::{Serialize, Deserialize};
use std::path::PathBuf;
use std::process::exit;
use reqwest::Url;
use control_cli::control::code;
use control_cli::control::code::{CommentedCode, get_common_values};
use control_cli::fs;
use control_cli::parser::SupportedLanguage;

#[derive(Debug, Parser)]
#[command(name = "control")]
#[command(bin_name = "control", version, author)]
#[command(about = "A CLI for code controls", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Set configuration variables
    Config(CliConfig),
    /// Run control commands
    Control(Control),
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
struct CliConfig {
    #[command(subcommand)]
    command: Option<ConfigCommands>,
}

#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
enum ConfigCommands {
    /// Set configuration variables
    Set(ConfigSet),
    /// Print the configuration path
    Path,
}

#[derive(Debug, Args)]
struct ConfigSet {
    /// Field to set
    #[arg(required = true)]
    field: String,
    /// Value to set
    #[arg(required = true)]
    value: String,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
struct Control {
    #[command(subcommand)]
    command: Option<ControlCommands>,
}

#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
enum ControlCommands {
    /// Operate on control code
    Code(ControlCode),
    /// Manage control parsers
    Parser(ControlParser),
    /// Print the control log
    Log(ControlLog),
}

#[derive(Debug, Args)]
struct ControlCode {
    /// Source code directory
    #[arg(required = true)]
    directory: PathBuf,
    /// Supported programming language
    #[arg(long, required = true)]
    lang: String,
    /// File extension(s)
    #[arg(long, required = true)]
    ext: Vec<String>,
    /// Output file path for control log
    #[arg(short, long, default_value = ".control-log")]
    output_file: PathBuf,
    /// Diff the generated hash with the one in the output file
    #[arg(long, action)]
    diff: bool,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
struct ControlParser {
    #[command(subcommand)]
    command: Option<ControlParserCommands>,
}

#[derive(Debug, Subcommand)]
#[command(arg_required_else_help = true)]
enum ControlParserCommands {
    /// Download a parser
    Download(ControlParserDownload),
}

#[derive(Debug, Args)]
struct ControlParserDownload {
    /// Supported programming language
    #[arg(required = true)]
    lang: String,
}

#[derive(Debug, Args)]
struct ControlLog {
    /// File path for control log
    #[arg(short, long, default_value = ".control-log")]
    log_path: PathBuf,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct Config {
    host: String,
    download_path: String,
}

fn main() -> Result<(), std::io::Error> {
    let args = Cli::parse();
    let mut config: Config = confy::load("control", "config").unwrap();

    match args.command {
        Commands::Config(cli_config) => {
            let config_cmd = cli_config.command.unwrap();
            match config_cmd {
                ConfigCommands::Set(set) => {
                    let field = set.field;
                    let value = set.value;
                    match field.as_str() {
                        "host" => {
                            config.host = value;
                        }
                        "download_path" => {
                            config.download_path = value;
                        }
                        _ => clap::Error::raw(clap::error::ErrorKind::Io, format!("Invalid field: {}", field)).exit()

                    }
                    confy::store("control", "config", config).unwrap();
                },
                ConfigCommands::Path => {
                    let config_path = confy::get_configuration_file_path("control", "config").unwrap();
                    println!("{}", config_path.to_str().unwrap());
                }
            }
        }
        Commands::Control(control) => {
            let control_cmd = control.command.unwrap();
            match control_cmd {
                ControlCommands::Parser(parser) => {
                    let parser_cmd = parser.command.unwrap();
                    match parser_cmd {
                        ControlParserCommands::Download(cmd) => {
                            let storage_path = match confy::get_configuration_file_path("control", "config") {
                                Ok(path) => {
                                    match path.parent() {
                                        Some(path) => {
                                            path.join(format!("libparser_{}.{}", cmd.lang, env::consts::DLL_EXTENSION))
                                        }
                                        None => clap::Error::raw(clap::error::ErrorKind::Io, "Could not get parent directory of config file").exit()
                                    }
                                }
                                Err(err) => clap::Error::raw(clap::error::ErrorKind::Io, err).exit()
                            };

                            match fs::read_file(storage_path.clone()) {
                                Some(_) => {
                                    println!("Parser already exists");
                                }
                                None => {
                                    let arch ;
                                    if env::consts::OS == "windows" {
                                        arch = "x86_64-pc-windows-msvc"
                                    } else if env::consts::OS == "macos" {
                                        arch = "x86_64-apple-darwin"
                                    } else {
                                        arch = "x86_64-unknown-linux-gnu"
                                    }
                                    if env::consts::OS == "windows" {
                                        let filename = format!("libparser_{}-{}.zip", cmd.lang, arch);
                                        let url = Url::parse(&config.host).unwrap().join(&config.download_path).unwrap().join(&filename).unwrap();
                                        match reqwest::blocking::get(url) {
                                            Ok(response) => {
                                                let mut archive = File::create(&filename).unwrap();
                                                match response.bytes() {
                                                    Ok(bytes) => {
                                                        archive.write(&bytes).unwrap();
                                                    }
                                                    Err(err) => clap::Error::raw(clap::error::ErrorKind::Io, err).exit()
                                                };
                                                zip_extract::extract(Cursor::new(&filename), &storage_path, true).unwrap();
                                                remove_file(&filename).unwrap();
                                            }
                                            Err(err) => clap::Error::raw(clap::error::ErrorKind::Io, err).exit()
                                        };
                                    } else {
                                        let filename = format!("libparser_{}-{}.tar.gz", cmd.lang, arch);
                                        let url = Url::parse(&config.host).unwrap().join(&config.download_path).unwrap().join(&filename).unwrap();
                                        match reqwest::blocking::get(url) {
                                            Ok(response) => {
                                                let mut archive = File::create(&filename).unwrap();
                                                match response.bytes() {
                                                    Ok(bytes) => {
                                                        archive.write(&bytes).unwrap();
                                                    }
                                                    Err(err) => clap::Error::raw(clap::error::ErrorKind::Io, err).exit()
                                                };
                                                let tar_gz = File::open(&filename).unwrap();
                                                let tar = flate2::read::GzDecoder::new(tar_gz);
                                                let mut archive = tar::Archive::new(tar);
                                                archive.unpack(&storage_path)?;
                                                remove_file(&filename).unwrap();
                                            }
                                            Err(err) => clap::Error::raw(clap::error::ErrorKind::Io, err).exit()
                                        };
                                    }
                                },
                            }

                        }
                    }
                },
                ControlCommands::Code(code) => {
                    if code.diff {
                        let decompressed = fs::decompress_file(code.output_file);

                        let mut old_commented_code_vec: Vec<CommentedCode> = bincode::deserialize(&decompressed).unwrap();
                        let mut new_commented_code_vec: Vec<CommentedCode> = code::get_control_commented_code(code.directory, SupportedLanguage::from(code.lang).language(), code.ext);
                        let matching: Vec<CommentedCode> = get_common_values(&old_commented_code_vec, &new_commented_code_vec);
                        old_commented_code_vec.retain(|x| !matching.contains(x));
                        new_commented_code_vec.retain(|x| !matching.contains(x));

                        if old_commented_code_vec.is_empty() && new_commented_code_vec.is_empty() {
                            println!("No changes detected.");
                        } else {
                            let mut output = String::new();
                            for commented_code in old_commented_code_vec {
                                output.push_str(&(format!("\t- {:?};{:?}:{:?}\n", commented_code.get_path(), commented_code.get_start().row + 1, commented_code.get_end().row + 1)));
                            }
                            for commented_code in new_commented_code_vec {
                                output.push_str(&format!("\t+ {:?};{:?}:{:?}\n", commented_code.get_path(), commented_code.get_start().row + 1, commented_code.get_end().row + 1));
                            }
                            print!("Changes Detected!\n {}", output);

                            exit(1);
                        }
                    } else {
                        let commented_code = code::get_control_commented_code(code.directory, SupportedLanguage::from(code.lang).language(), code.ext);
                        if commented_code.is_empty() {
                            clap::Error::raw(clap::error::ErrorKind::Io, "No commented code found.\n").exit();
                        } else {
                            let file = File::create(code.output_file.to_str().unwrap().to_string()).unwrap();
                            let mut file_writer = brotli::CompressorWriter::new(file, 4096, 5, 22);
                            match bincode::serialize_into(&mut file_writer, &commented_code) {
                                Ok(_) => {
                                    println!("{} generated.", code.output_file.to_str().unwrap().to_string());
                                }
                                Err(err) => {
                                    clap::Error::raw(clap::error::ErrorKind::Io, format!("Error generating file: {}", err)).exit();
                                }
                            }
                        }
                    }
                }
                ControlCommands::Log(log) => {
                    let decompressed = fs::decompress_file(log.log_path);
                    let commented_code_vec: Vec<CommentedCode> = bincode::deserialize(&decompressed).unwrap();
                    match serde_json::to_string_pretty(&commented_code_vec) {
                        Ok(content) => {
                            println!("{}", content);
                        }
                        Err(_) => {
                            clap::Error::raw(clap::error::ErrorKind::Io, "Error parsing log file.").exit();
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
