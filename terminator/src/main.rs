use std::error::Error;
use std::env;
use std::fmt::{Display, Formatter};
use std::io::{stdin, stdout, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::process::exit;
use rusqlite::{Connection, Statement};

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

#[derive(Debug)]
struct NoSuchUser;

impl Display for NoSuchUser {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "NoSuchUser Error - no user can be found")
    }
}

impl Error for NoSuchUser {}

#[derive(Debug)]
struct BadPassword;

impl Display for BadPassword {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Password must be at least 10 characters long, use uppercase and lowercase letters,\
        and use at least one symbol.")
    }
}

impl Error for BadPassword {}

#[derive(Debug)]
struct UserError {
    source: Box<dyn Error>,
}

impl Display for UserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "An error occurred with regards to the user. {}", self.source.deref())
    }
}

impl Error for UserError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

fn main() {
    // Establish the path to the database for querying whether this User exists
    let mut db: PathBuf = env::current_dir().expect("Couldn't obtain cwd.");
    db.pop();
    db = db.join("/assets/terminator.db");
    let conn: Connection = Connection::open(&db).expect("Couldn't connect to db.");

    let _user = select_user(&conn);
}

/// Selects a user for playing the game.  If there are no users already established, then the user
/// will have to create a new one.
fn select_user(conn: &Connection) -> User {
    // Print the logo
    println!("{}", LOGO);

    // The user that will be playing the game; either a current User or a new User
    let user: User;

    let mut temp: String = String::new();
    loop {
        temp.clear();
        println!("Are you a new user or current user?");
        print!("Enter 'c' for current user or 'n' for new user or 'q' to quit: ");
        stdout().flush().expect("Unable to flush stdout.");
        stdin().read_line(&mut temp).expect("Unable to read stdin.");

        let choice = temp.trim();

        match choice {
            "c" => {
                temp.clear();
                print!("Enter your username: ");
                stdout().flush().expect("Unable to flush stdout...");

                if let Some(usr) = find_user_by_username(&conn, temp.trim()) {
                    user = usr;
                    break;
                } else {
                    println!("Unable to get current user. Please create a new one or try again.");
                }
            },
            "n" => {
                if let Ok(usr) = create_new_user(&conn) {
                    user = usr;
                    break;
                } else {
                    println!("Unable to create a new user. Please try again.");
                }
            },
            "q" => exit(0),
            _ => println!("Please make a valid selection!"),
        }
    }

    user
}

fn find_user_by_username(conn: &Connection, username: &str) -> Result<User, UserError> {
    let mut stmt: Statement = conn.prepare(
        "SELECT Username, Password FROM Users WHERE Username = ?1"
    ).expect("Statement unsuccessfully created...");

    let query = stmt.query_row(&[username], |row| {
        Ok(User {
            username: row.get(0)?,
            password: row.get(1)?,
        })
    });

    if query.is_err() {
        Err(UserError {source: Box::new(NoSuchUser)})
    } else {
        Ok(query.unwrap())
    }
}

fn create_new_user(conn: &Connection) -> Result<User, UserError> {
    let username: String;
    let password: String;
    println!("Creating a new user...");
    let mut temp: String = String::new();
    loop {
        temp.clear();
        print!("Enter your desired username: ");
        stdout().flush().expect("Unable to flush stdout...");
        stdin().read_line(&mut temp).expect("Unable to read stdin...");

        let name = temp.trim();

        if find_user_by_username(&conn, name).is_ok() {
            println!("Username already exists; please pick a new username");
        } else {
            username = String::from(name);
            password = create_valid_password();
            break;
        }
    }

    let stmt = conn.prepare(
        "INSERT INTO Users (Username, Password) VALUES (?1, ?2)"
    ).expect("Unable to craft statement");

    // TODO: insert stmt into db with args.
}

fn create_valid_password() -> String {
    let mut temp: String = String::new();
    // TODO:
    temp
}