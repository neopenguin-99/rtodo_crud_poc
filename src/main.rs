use std::thread;
use std::time::Duration;

use console::Term;
use rusqlite::{params, Connection, Result};

use clap::{crate_authors, crate_version, value_parser, Arg, ArgMatches, Command};

fn parse_args() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let arg_matches: ArgMatches = Command::new("clap")
        .version(crate_version!())
        .author(crate_authors!("\n"))
            .subcommands(
            [
                Command::new("add").about("add new item to todo").arg(Arg::new("add")),
                Command::new("done").about("set existing item to done").arg(Arg::new("done")),
                Command::new("remove").about("remove existing item from todo list").arg(Arg::new("remove")),
            ])
            .arg(Arg::new("files")
            .num_args(0..)
            .value_parser(value_parser!(String))
        )
        .try_get_matches()?;
    match arg_matches.subcommand() {
        Some(("add", val)) => todo!("add"),
        Some(("done", val)) => todo!("done"),
        Some(("remove", val)) => todo!("remove"),
        Some((_, _)) => panic!("Please provide a subcommand"),
        None => todo!("list")
    }
}

#[derive(Debug)]
struct Item {
    id: i32,
    note: String,
    is_done: bool
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let conn = Connection::open_in_memory()?;

    conn.execute("CREATE TABLE item (
        id INTEGER PRIMARY KEY,
        note TEXT NOT NULL,
        is_done NOT NULL
    )", ())?;


    // term.write_line("Hello World!")?;
    // thread::sleep(Duration::from_millis(2000));
    // term.clear_line()?;
    let args = parse_args()?;
    Ok(())
}


