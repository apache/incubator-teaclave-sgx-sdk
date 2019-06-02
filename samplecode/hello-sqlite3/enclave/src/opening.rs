use std::default::Default;
use std::io::Write;
use std::prelude::v1::*;
use std::vec::Vec;

use sqlite3::{
    Access,
    DatabaseConnection,
    QueryFold,
    ResultRowAccess,
    SqliteResult,
    StatementUpdate,
};
use sqlite3::access;
use sqlite3::access::flags::Flags;

pub fn opening() {
    let args : Vec<String> = Vec::new();
    let usage = "sqlite";


    let cli_access = {
        let ok = |flags, dbfile| Some(access::ByFilename { flags: flags, filename: dbfile });

        let arg = |n| {
            if args.len() > n { Some(args[n].as_ref()) }
            else { None }
        };

        match (arg(1), arg(2)) {
            (Some("-r"), Some(dbfile))
            => ok(Flags::OPEN_READONLY, dbfile),
            (Some(dbfile), None)
            => ok(Default::default(), dbfile),
            (_, _)
            => None
        }
    };

    println!("test_openings success!");

    fn use_access<A: Access>(access: A) -> SqliteResult<Vec<Person>> {
        let mut conn = try!(DatabaseConnection::new(access));
        make_people(&mut conn)
    }


    fn lose(why: &str) {
        // FIXME: Set the exit status once that is stabilized
        let stderr = std::io::stderr();
        let mut stderr_lock = stderr.lock();
        stderr_lock.write_fmt(format_args!("{}", why)).unwrap()
    }

    match cli_access {
        Some(a) => match use_access(a) {
            Ok(x) => println!("Ok: {:?}", x),
            Err(oops) => lose(format!("oops!: {:?}", oops).as_ref())
        },
        None => lose(usage)
    }
}


#[derive(Debug, Clone)]
struct Person {
    id: i32,
    name: String,
}

fn make_people(conn: &mut DatabaseConnection) -> SqliteResult<Vec<Person>> {
    conn.exec("CREATE TABLE person (
                 id              SERIAL PRIMARY KEY,
                 name            VARCHAR NOT NULL
               )")?;

    {
        let mut tx = conn.prepare("INSERT INTO person (id, name)
                           VALUES (0, 'Dan')")?;
        let changes = tx.update(&[])?;
        assert_eq!(changes, 1);
    }

    let mut stmt = conn.prepare("SELECT id, name FROM person")?;

    let snoc = |x, mut xs: Vec<_>| { xs.push(x); xs };

    let ppl = stmt.query_fold(
        &[], vec!(), |row, ppl| {
            Ok(snoc(Person {
                id: row.get(0),
                name: row.get(1)
            }, ppl))
        })?;
    Ok(ppl)
}

// Local Variables:
// flycheck-rust-library-path: ("../target")
// End:
