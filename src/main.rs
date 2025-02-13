use std::thread;
use std::time::Duration;

use console::Term;
use colored::Colorize;
use rusqlite::{params, Connection, OpenFlags, Result};

use clap::{crate_authors, crate_version, value_parser, Arg, ArgMatches, Command};

fn parse_args() -> Result<(&'static dyn Fn(&Connection, ItemCommand)->Result<usize, Box<dyn std::error::Error>>, ItemCommand), Box<dyn std::error::Error>> {
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
        Some(("add", val)) => Ok((&insert_into_item_table, ItemCommand::Add(val.get_many::<String>("do").unwrap().map(|s| s.as_str()).collect()))),
        Some(("done", val)) => Ok((&set_item_as_done, ItemCommand::Done(*val.get_one::<usize>("").unwrap()))),
        Some(("remove", val)) => Ok((&remove_item, ItemCommand::Remove(*val.get_one::<usize>("").unwrap()))),
        Some((_, _)) => panic!("Please provide a subcommand"),
        None => Ok((&list_items, ItemCommand::List))
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

impl Item {
    fn new(note: impl Into<String>) -> Item {
        Item {
            id: 99999999, // todo think of something better
            is_done: false,
            note: Into::into(note)
        }
    }
}

fn create_item_table(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute("CREATE TABLE item (
        id INTEGER PRIMARY KEY,
        note TEXT NOT NULL,
        is_done NOT NULL
    )", ())?;
    Ok(())
}

fn insert_into_item_table(conn: &Connection, item: ItemCommand) -> Result<usize, Box<dyn std::error::Error>> {
    let note = match item {
        ItemCommand::Add(note) => note,
        _ => panic!("note not in item command")
    };
    let rows = conn.execute("INSERT INTO item (note, is_done) VALUES (?1, ?2)", (note, false));
    Ok(rows?)
}

fn set_item_as_done(conn: &Connection, item: ItemCommand) -> Result<usize, Box<dyn std::error::Error>> {
    let id = match item {
        ItemCommand::Done(id) => id,
        _ => panic!("id not in command to set item as done")
    };
    let rows = conn.execute("UPDATE item SET is_done = ?1 WHERE id = ?2", (true, id));
    Ok(rows?)
}

fn remove_item(conn: &Connection, item: ItemCommand) -> Result<usize, Box<dyn std::error::Error>> {
    let id = match item {
        ItemCommand::Remove(id) => id,
        _ => panic!("id not in command to remove")
    };
    let rows = conn.execute("DELETE FROM item WHERE id = ?1", ((id),));
    Ok(rows?)
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
            println!("{}", item.note.strikethrough());
        }
        else {
            println!("{}", item.note);
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
    let (func, command) = parse_args()?;
    let _ = func(&conn.unwrap(), command)?;
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
        let rows = insert_into_item_table(&conn, ItemCommand::Add(String::from("do laundry")))?;
        assert_eq!(rows, 1);
        Ok(())
    }

}


