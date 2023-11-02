use std::env;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;
use rusqlite::{Connection, Error, ErrorCode};

const LOGO: &str = "

████████╗███████╗██████╗ ███╗   ███╗██╗███╗   ██╗ █████╗ ████████╗ ██████╗ ██████╗
╚══██╔══╝██╔════╝██╔══██╗████╗ ████║██║████╗  ██║██╔══██╗╚══██╔══╝██╔═══██╗██╔══██╗
   ██║   █████╗  ██████╔╝██╔████╔██║██║██╔██╗ ██║███████║   ██║   ██║   ██║██████╔╝
   ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║██║██║╚██╗██║██╔══██║   ██║   ██║   ██║██╔══██╗
   ██║   ███████╗██║  ██║██║ ╚═╝ ██║██║██║ ╚████║██║  ██║   ██║   ╚██████╔╝██║  ██║
   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝

";

struct User {
    username: String,
    password: String,
}

fn main() {
    println!("{LOGO}");
}

/// Selects a user for playing the game.  If there are no users already established, then the user
/// will have to create a new one.
fn select_user() -> User {
    // The user that will be playing the game; either a current User or a new User
    let user: User;

    // Establish the path to the database for querying whether this User exists
    let mut db: PathBuf = env::current_dir().expect("Couldn't obtain cwd.");
    db.pop();
    db = db.join("/assets/terminator.db");
    let conn: Connection = Connection::open(&db).expect("Couldn't connect to db.");

    let mut temp: String = String::new();
    let mut username: String = String::new();
    let mut password: String = String::new();

    loop {
        temp.clear();
        print!("Enter your username: ");
        stdout().flush().expect("Unable to flush stdout.");
        stdin().read_line(&mut temp).expect("Unable to read stdin.");


        // TODO: check if current user or create a new user
    }

    user
}

fn find_user_by_username(conn: &Connection, username: &str) -> Result<Option<User>, Error> {
    let mut stmt = conn.prepare("SELECT Username FROM Users WHERE Username = ?1")?;
    let user_iter = stmt.query_map(&[username], |row| {
        Ok(User {
            username: row.get(0)?,
            password: row.get(1)?,
        })
    })?;

    for user_result in user_iter {
        return user_result.map(Some);
    }

    Ok(None)
}

fn create_new_user(conn: &Connection, username: &str) -> Result<User, ErrorCode> {
    let mut temp: String = String::new();
    loop {
        temp.clear();
        println!("Creating a new user...");
        print!("Enter your password: ");
        stdout().flush().expect("Unable to flush stdout...");

    }
}