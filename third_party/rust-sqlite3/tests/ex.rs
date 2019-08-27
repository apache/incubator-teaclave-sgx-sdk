extern crate time;
extern crate sqlite3;

use time::Timespec;


use sqlite3::{
    DatabaseConnection,
    Query,
    ResultRow,
    ResultRowAccess,
    SqliteResult,
    StatementUpdate,
};

#[derive(Debug, Clone)]
struct Person {
    id: i32,
    name: String,
    time_created: Timespec,
    // TODO: data: Option<Vec<u8>>
}

pub fn main() {
    match io() {
        Ok(ppl) => println!("Found people: {:?}", ppl),
        Err(oops) => panic!(oops)
    }

    println!("ex");
}

fn io() -> SqliteResult<Vec<Person>> {
    let mut conn = try!(DatabaseConnection::in_memory());

    try!(conn.exec("CREATE TABLE person (
                 id              SERIAL PRIMARY KEY,
                 name            VARCHAR NOT NULL,
                 time_created    TIMESTAMP NOT NULL
               )"));

    let me = Person {
        id: 0,
        name: "Dan".to_owned(),
        time_created: time::get_time(),
    };
    {
        let mut tx = try!(conn.prepare("INSERT INTO person (name, time_created)
                           VALUES ($1, $2)"));
        let changes = try!(tx.update(&[&me.name, &me.time_created]));
        assert_eq!(changes, 1);
    };

    let mut stmt = try!(conn.prepare("SELECT id, name, time_created FROM person"));

    let to_person = |row: &mut ResultRow| Ok(
        Person {
            id: row.get("id"),
            name: row.get("name"),
            time_created: row.get(2)
        });
    let ppl = try!(stmt.query(&[], to_person));
    ppl.collect()
}
