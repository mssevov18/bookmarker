use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self};
use std::path::Path;
// use std::path::PathBuf;
use terminal_size::{terminal_size, Width};

#[derive(Parser, Debug)]
#[command(name = "b")]
#[command(version)]
#[command(about="Navigate using bookmarks", long_about=None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        key: String,
        name: String,
        path: Option<String>,
    },
    Remove {
        key: String,
    },
    Quick {
        key: String,
    },
    All,     //List all bookmarks (Short format)
    AllLong, //List all bookmarks (Long format)
    AllTop,  //List top5 bookmarks
    Purge,   //Remove all bookmarks (delete the file)
    Clean,   //Remove all unreachable bookmarks
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Bookmark {
    name: String,
    path: String,
    key: String,
    uses: usize,
}

fn save_bookmarks(bookmarks: &Vec<Bookmark>, path: &Path) -> std::io::Result<()> {
    let json =
        serde_json::to_string_pretty(bookmarks).expect("Failed to serialize bookmarks to JSON");
    fs::write(path, json)?;
    Ok(())
}

fn load_bookmarks(path: &Path) -> std::io::Result<Vec<Bookmark>> {
    if path.exists() {
        let data = fs::read_to_string(path)?;
        let bookmarks: Vec<Bookmark> =
            serde_json::from_str(&data).expect("Failed to deserialize bookmarks from JSON");
        Ok(bookmarks)
    } else {
        Ok(vec![])
    }
}

fn main() {
    let cli = Cli::parse();

    // let db_path = PathBuf::from("~/.config/bookmarker/bookmarks.json");
    let db_path = dirs::config_dir()
        .unwrap_or_else(|| {
            eprintln!("Could not find config directory.");
            std::process::exit(1);
        })
        .join("bookmarker/bookmarks.json");
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let mut bookmarks = load_bookmarks(&db_path).unwrap_or_else(|e| {
        eprintln!("Error reading bookmarks: {e}");
        std::process::exit(1);
    });

    match &cli.command {
        Some(Commands::Add { key, name, path }) => {
            if bookmarks.iter().any(|b| b.key == *key) {
                eprintln!("A bookmark with key '{}' already exists.", key);
                std::process::exit(1);
            }

            let actual_path = match path {
                Some(p) => p.clone(),
                None => std::env::current_dir()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            };

            let new_bookmark = Bookmark {
                name: name.clone(),
                key: key.clone(),
                path: actual_path,
                uses: 0,
            };

            bookmarks.push(new_bookmark);
            save_bookmarks(&bookmarks, &db_path).unwrap_or_else(|e| {
                eprintln!("Error saving bookmark: {e}");
                std::process::exit(1);
            });

            println!("Bookmark added: [{}]#{}", key, name);
        }

        Some(Commands::Remove { key }) => {
            let before = bookmarks.len();

            bookmarks.retain(|b| b.key != *key);

            if bookmarks.len() < before {
                save_bookmarks(&bookmarks, &db_path).unwrap_or_else(|e| {
                    eprintln!("Error saving bookmarks: {e}");
                    std::process::exit(1);
                });
                println!("Bookmark with key '{}' deleted", key);
            } else {
                println!("No bookmarks found with key '{}'", key);
            }
        }

        Some(Commands::Quick { key }) => {
            let maybe_bm = bookmarks.iter_mut().find(|b| b.key == *key);
            if let Some(bm) = maybe_bm {
                bm.uses += 1;
                let path_to_print = bm.path.clone();

                if let Err(e) = save_bookmarks(&bookmarks, &db_path) {
                    eprintln!("Could not update usage count: {e}");
                    std::process::exit(1);
                }

                println!("{}", path_to_print);
            } else {
                eprintln!("No bookmark found for key '{}'", key);
                std::process::exit(2);
            }
        }

        Some(Commands::All) => {
            if bookmarks.is_empty() {
                return;
            }

            let mut line = String::new();
            let term_width = match terminal_size() {
                Some((Width(w), _)) => w as usize,
                None => 80, //fallback
            };

            for bm in &bookmarks {
                let entry = format!("[{}]{}  ", bm.key, bm.name);
                if line.len() + entry.len() > term_width {
                    println!("{}", line.trim_end());
                    line.clear();
                }
                line.push_str(&entry);
            }

            if !line.is_empty() {
                println!("{}", line.trim_end());
            }
        }

        Some(Commands::AllLong) => {
            if bookmarks.is_empty() {
                return;
            }

            for bm in &bookmarks {
                println!("[{}] {} ({} uses) - {}", bm.key, bm.name, bm.uses, bm.path);
            }
        }

        Some(Commands::Purge) => {
            if db_path.exists() {
                match std::fs::remove_file(&db_path) {
                    Ok(_) => println!("All bookmarks deleted. (:"),
                    Err(e) => {
                        eprintln!("Failed to delete bookmark file: {e}");
                        std::process::exit(1);
                    }
                }
            } else {
                println!("No bookmark file found - nothing to purge. ):")
            }
        }

        Some(Commands::Clean) => {
            if bookmarks.is_empty() {
                println!("No bookmarks to clean.");
                return;
            }

            let before = bookmarks.len();

            bookmarks.retain(|b| {
                let exists = std::path::Path::new(&b.path).exists();
                if !exists {
                    println!("Removed: [{}]{} - missing path '{}'", b.key, b.name, b.path);
                }
                exists
            });

            if bookmarks.len() < before {
                save_bookmarks(&bookmarks, &db_path).unwrap_or_else(|e| {
                    eprintln!("Failed to save cleaned bookmarks: {e}");
                    std::process::exit(1);
                });
                println!("Cleaned {} invalid bookmark(s).", before - bookmarks.len());
            } else {
                println!("All bookmarks are valid. Nothing to clean.");
            }
        }

        Some(Commands::AllTop) => {
            if bookmarks.is_empty() {
                println!("No bookmarks available. Try help, add.");
                std::process::exit(0);
            }

            let mut sorted = bookmarks.clone();
            sorted.sort_by(|a, b| b.uses.cmp(&a.uses));

            println!("Most used 5 bookmarks:");

            let display_count = usize::min(5, sorted.len());
            let mut line = String::new();

            for bm in sorted.iter().take(display_count) {
                line.push_str(format!("[{}]{}  ", bm.key, bm.name).as_str());
            }

            if !line.is_empty() {
                println!("{}", line.trim_end());
            }
        }

        None => {
            if bookmarks.is_empty() {
                println!("No bookmarks available. Try help, add.");
                std::process::exit(0);
            }

            let mut sorted = bookmarks.clone();
            sorted.sort_by(|a, b| b.uses.cmp(&a.uses));

            println!("Most used 5 bookmarks:");

            let display_count = usize::min(5, sorted.len());
            let mut line = String::new();

            for bm in sorted.iter().take(display_count) {
                line.push_str(format!("[{}]{}  ", bm.key, bm.name).as_str());
            }

            if !line.is_empty() {
                println!("{}", line.trim_end());
            }

            print!("Choose key: ");

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            let maybe_bm = bookmarks.iter_mut().find(|b| b.key == input);
            if let Some(bm) = maybe_bm {
                bm.uses += 1;
                let tmppath = bm.path.clone();

                save_bookmarks(&bookmarks, &db_path).unwrap_or_else(|e| {
                    eprintln!("Could not save usage update: {e}");
                    std::process::exit(1);
                });

                println!("{}", tmppath); //print for the shell wrapper
            } else {
                eprintln!("No bookmark found for key '{}'", input);
                std::process::exit(2);
            }
        }
    }
}
