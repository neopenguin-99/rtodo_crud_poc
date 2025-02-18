#![feature(assert_matches)]
use std::thread;
use std::time::Duration;

use console::Term;
use colored::Colorize;
use rusqlite::{Connection, OpenFlags, Result};
use std::os::unix::fs;
use std::os::unix::io;

use clap::{crate_authors, crate_version, value_parser, Arg, ArgMatches, Command};

fn parse_args() -> Result<(ItemCommand, String), Box<dyn std::error::Error>> {
    let default_db_file_name = String::from("todo.db");
    let mut arg_matches: ArgMatches = Command::new("clap")
        .allow_external_subcommands(true)
        .version(crate_version!())
        .author(crate_authors!("\n"))
            .arg(Arg::new("d")
            .value_parser(value_parser!(String))
            .short('d')
            .help("specify custom database file to read and write from. If the database file entered does not exist, then it is created. If this option is not specified, then the default file name will be used, todo.db"))
            .subcommands(
            [
                Command::new("add").about("add new item to todo").arg(Arg::new("add")),
                Command::new("done").about("set existing item to done").arg(Arg::new("done").value_parser(value_parser!(usize))),
                Command::new("remove").about("remove existing item from todo list").arg(Arg::new("remove").value_parser(value_parser!(usize))),
            ])
        .try_get_matches()?;

    let db_file_name: String = arg_matches.remove_one::<String>("d").unwrap_or(default_db_file_name.clone());

    match arg_matches.subcommand() {
        Some(("add", val)) => Ok((ItemCommand::Add(val.get_many::<String>("add").unwrap().map(|s| s.as_str()).collect()), db_file_name)),
        Some(("done", val)) => Ok((ItemCommand::Done(*val.get_one::<usize>("done").unwrap()), db_file_name)),
        Some(("remove", val)) => Ok((ItemCommand::Remove(*val.get_one::<usize>("remove").unwrap()), db_file_name)),
        Some((_, _)) if db_file_name == default_db_file_name => panic!("Please provide a subcommand"),
        Some((_, _)) => Ok((ItemCommand::List, db_file_name)),
        None => Ok((ItemCommand::List, db_file_name))
    }

}

#[derive(Debug)]
struct Item {
    id: i32,
    note: String,
    is_done: bool
}

enum ItemCommand {
    Add(String),
    Done(usize),
    Remove(usize),
    List
}

fn create_item_table(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute("CREATE TABLE item (
        id INTEGER PRIMARY KEY,
        note TEXT NOT NULL,
        is_done NOT NULL
    )", ())?;
    Ok(())
}

fn insert_into_item_table(conn: &Connection, note: String) -> Result<usize, Box<dyn std::error::Error>> {
    let rows = conn.execute("INSERT INTO item (note, is_done) VALUES (?1, ?2)", (note, false));
    Ok(rows?)
}

fn set_item_as_done(conn: &Connection, id: usize) -> Result<usize, Box<dyn std::error::Error>> {
    let rows = conn.execute("UPDATE item SET is_done = ?1 WHERE id = ?2", (true, id));
    Ok(rows?)
}

fn remove_item(conn: &Connection, id: usize) -> Result<usize, Box<dyn std::error::Error>> {
    let rows = conn.execute("DELETE FROM item WHERE id = ?1", ((id),));
    Ok(rows?)
}

fn exec(conn: &Connection, item: ItemCommand) -> Result<usize, Box<dyn std::error::Error>> {
    match item {
        ItemCommand::List => list_items(conn, item),
        ItemCommand::Add(item) => insert_into_item_table(conn, item),
        ItemCommand::Done(item) => set_item_as_done(conn, item),
        ItemCommand::Remove(item) => remove_item(conn, item),
    }
}

fn list_items(conn: &Connection, item: ItemCommand) -> Result<usize, Box<dyn std::error::Error>> {
    _ = item;
    let mut stmt = conn.prepare("SELECT id, note, is_done FROM item")?;
    let item_iter = stmt.query_map([], |row| {
        Ok(Item {
            id: row.get(0)?,
            note: row.get(1)?,
            is_done: row.get(2)?
        })
    })?;
    let mut rows = 0;
    for item in item_iter {
        let item = item.unwrap();
        if item.is_done {
            println!("{}: {}", item.id, item.note.strikethrough());
        }
        else {
            println!("{}: {}", item.id, item.note);
        }
        rows = rows + 1;
    }
    Ok(rows)
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    let (command, database_path) = parse_args()?;
    let mut conn = Connection::open_with_flags(&database_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_URI
        | OpenFlags::SQLITE_OPEN_NO_MUTEX);

    if conn.is_err() {
        conn = Ok(Connection::open(database_path).unwrap());
        create_item_table(&conn.as_ref().unwrap())?;
    }

    // term.write_line("Hello World!")?;
    // thread::sleep(Duration::from_millis(2000));
    // term.clear_line()?;

    let _ = exec(&conn.unwrap(), command)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;
    use rusqlite::named_params;
    use std::process::Command;
    use assert_cmd::prelude::*;

    use tempfile::TempDir;

    use super::*;

    fn create_empty_item_table_for_test() -> Result<Connection, Box<dyn std::error::Error>> {
        let conn = Connection::open_in_memory()?;

        conn.execute("CREATE TABLE item (
            id INTEGER PRIMARY KEY,
            note TEXT NOT NULL,
            is_done NOT NULL
        )", ())?;


        Ok(conn)
    }

    fn add_item_to_table_for_test(conn: &Connection, note: impl Into<String>, is_done: bool) -> Result<usize, Box<dyn std::error::Error>> {
        let rows = conn.execute("INSERT INTO item (note, is_done) VALUES (?1, ?2)", (Into::into(note), is_done));
        Ok(rows?)
    }
    

    fn get_note_by_id_for_test(conn: &Connection, id: usize) -> Result<Item, i32> {
        let mut receiver = conn
            .prepare("SELECT id, note, is_done FROM item WHERE id = :id;")
            .expect("receiver failed");
        let mut rows = receiver
            .query(named_params!{ ":id": id })
            .expect("rows failed");
        while let Some(row) = rows.next().expect("while row failed") {
            return Ok(Item {
                id: row.get(0).expect("get id failed"),
                note: row.get(1).expect("get note failed"),
                is_done: row.get(2).expect("get is_done failed")
            });
        }
        return Err(-1);
    }

    #[test]
    fn test_insert() -> Result<(), Box<dyn std::error::Error>> {
        let conn = create_empty_item_table_for_test()?;
        let rows = exec(&conn, ItemCommand::Add(String::from("do laundry")))?;
        assert_eq!(rows, 1);
        Ok(())
    }

    #[test]
    fn test_done() -> Result<(), Box<dyn std::error::Error>> {
        let conn = create_empty_item_table_for_test()?;
        _ = add_item_to_table_for_test(&conn, "get milk", false)?;
        let rows = exec(&conn, ItemCommand::Done(1))?;
        assert_eq!(rows, 1);
        let res = get_note_by_id_for_test(&conn, 1);
        assert_matches!(res, Ok(x) if x.is_done == true);
        Ok(())
    }

    #[test]
    fn test_list() -> Result<(), Box<dyn std::error::Error>> {
        let conn = create_empty_item_table_for_test()?;
        _ = add_item_to_table_for_test(&conn, "do laundry", true)?;
        _ = add_item_to_table_for_test(&conn, "get milk", false)?;
        let rows = exec(&conn, ItemCommand::List)?;
        assert_eq!(rows, 2);
        Ok(())
    }

    #[test]
    fn test_delete() -> Result<(), Box<dyn std::error::Error>> {
        let conn = create_empty_item_table_for_test()?;
        assert_eq!(add_item_to_table_for_test(&conn, "do laundry", true)?, 1);
        assert_eq!(add_item_to_table_for_test(&conn, "get milk", false)?, 1);

        let num_rows_affected = exec(&conn, ItemCommand::Remove(1))?;
        assert_eq!(num_rows_affected, 1);
        let row_one_res = get_note_by_id_for_test(&conn, 1);
        assert_matches!(row_one_res, Err(_));
        let row_two_res = get_note_by_id_for_test(&conn, 2);
        assert_matches!(row_two_res, Ok(x) if x.note == String::from("get milk"));
        Ok(())
    }

    #[test]
    fn some_test() -> Result<(), Box<dyn std::error::Error>> {

        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        cmd.arg("add do laundry");

        let assertion = cmd.assert().try_success()?;
        Ok(())

    }

    #[test]
    fn test_db_creation() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().to_str().unwrap();
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        println!("{}", db_path);
        let args: String = format!("-d {}/mytodo.db", db_path);
        cmd.arg(args);

        let assertion = cmd.assert().try_success()?;
        // Teardown
        Ok(())
    }

    #[test]
    fn test_db_subcommands_passed_to_exec() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().to_str().unwrap();
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
        println!("{}", db_path);
        Ok(())
    }
}


