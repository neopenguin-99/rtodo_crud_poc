use std::thread;
use std::time::Duration;

use console::Term;
use rusqlite::{params, Connection, OpenFlags, Result};

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

fn create_item_table(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute("CREATE TABLE item (
        id INTEGER PRIMARY KEY,
        note TEXT NOT NULL,
        is_done NOT NULL
    )", ())?;
    Ok(())
}

fn insert_into_item_table(conn: &Connection, note: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let rows = conn.execute("INSERT INTO item (note, is_done) VALUES (?1, ?2)", (note, false));
    Ok(rows?)
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let database_path = "./todo.db";
    let mut conn = Connection::open_with_flags(database_path,
    OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_URI
        | OpenFlags::SQLITE_OPEN_NO_MUTEX);

    if conn.is_err() {
        conn = Ok(Connection::open(database_path).unwrap());
        create_item_table(&conn.unwrap())?;
    }




    // term.write_line("Hello World!")?;
    // thread::sleep(Duration::from_millis(2000));
    // term.clear_line()?;
    let args = parse_args()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::{NamedTempFile, Builder, TempDir};


    fn create_empty_item_table_for_test() -> Result<Connection, Box<dyn std::error::Error>> {
        let conn = Connection::open_in_memory()?;

        conn.execute("CREATE TABLE item (
            id INTEGER PRIMARY KEY,
            note TEXT NOT NULL,
            is_done NOT NULL
        )", ())?;

        Ok(conn)
    }

    #[test]
    fn test_insert() -> Result<(), Box<dyn std::error::Error>> {
        let conn = create_empty_item_table_for_test()?;
        let rows = insert_into_item_table(&conn, "do laundry")?;
        assert_eq!(rows, 1);
        Ok(())
    }

}


