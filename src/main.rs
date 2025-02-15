#![feature(assert_matches)]
use std::thread;
use std::time::Duration;

use console::Term;
use colored::Colorize;
use rusqlite::{params, Connection, OpenFlags, Result};

use clap::{crate_authors, crate_version, value_parser, Arg, ArgMatches, Command};

fn parse_args() -> Result<ItemCommand, Box<dyn std::error::Error>> {
    let arg_matches: ArgMatches = Command::new("clap")
        .allow_external_subcommands(true)
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
        Some(("add", val)) => Ok(ItemCommand::Add(val.get_many::<String>("do").unwrap().map(|s| s.as_str()).collect())),
        Some(("done", val)) => Ok(ItemCommand::Done(*val.get_one::<usize>("").unwrap())),
        Some(("remove", val)) => Ok(ItemCommand::Remove(*val.get_one::<usize>("").unwrap())),
        Some((_, _)) => panic!("Please provide a subcommand"),
        None => Ok(ItemCommand::List)
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
    let database_path = "./todo.db";
    let mut conn = Connection::open_with_flags(database_path,
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
    let command = parse_args()?;
    let _ = exec(&conn.unwrap(), command)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

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

    fn add_item_to_table_for_test(conn: &Connection, note: impl Into<String>, is_done: bool) -> Result<usize, Box<dyn std::error::Error>> {
        let rows = conn.execute("INSERT INTO item (note, is_done) VALUES (?1, ?2)", (Into::into(note), is_done));
        Ok(rows?)
    }
    

    fn get_note_by_id_for_test(conn: &Connection, id: usize) -> Result<(), Box<dyn std::error::Error>> {
        // let rows = stmt.query_row::<String, _>(&[(":id", id.to_string().as_str())], |row| row.get(0))?;
        // let res = stmt.query_row::<String, _>(("SELECT note FROM item WHERE id = :id", ), |r| r.get(0));
        // let res = stmt.query_row("SELECT note FROM item WHERE id = :id", |r| r.get(0));

        let sql = format!("SELECT note FROM item WHERE id = :{}", id);
        let _ = conn.query_row(&sql, [], |row| {
            println!("{:?}", row.get(0).expect("one"));
            Ok(())
        });
        let mut stmt = conn.prepare(&sql)?;

        Ok(())
        
    }

    #[test]
    fn test_insert() -> Result<(), Box<dyn std::error::Error>> {
        let conn = create_empty_item_table_for_test()?;
        let rows = exec(&conn, ItemCommand::Add(String::from("do laundry")))?;
        assert_eq!(rows, 1);
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

        let a = list_items(&conn, ItemCommand::List);
        println!("{:#?}", a);
        let num_rows_affected = exec(&conn, ItemCommand::Remove(1))?;
        let b = list_items(&conn, ItemCommand::List);
        println!("{:#?}", b);
        assert_eq!(num_rows_affected, 1);
        let res = get_note_by_id_for_test(&conn);
        assert_matches!(res, Err(_));
        Ok(())
    }
}


