use std::error::Error;
use std::env;
use std::fmt::{Display, Formatter};
use std::io::{stdin, stdout, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::process::exit;
use rusqlite::{Connection, Statement};
use crate::term_user::User;

const LOGO: &str = "

████████╗███████╗██████╗ ███╗   ███╗██╗███╗   ██╗ █████╗ ████████╗ ██████╗ ██████╗
╚══██╔══╝██╔════╝██╔══██╗████╗ ████║██║████╗  ██║██╔══██╗╚══██╔══╝██╔═══██╗██╔══██╗
   ██║   █████╗  ██████╔╝██╔████╔██║██║██╔██╗ ██║███████║   ██║   ██║   ██║██████╔╝
   ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║██║██║╚██╗██║██╔══██║   ██║   ██║   ██║██╔══██╗
   ██║   ███████╗██║  ██║██║ ╚═╝ ██║██║██║ ╚████║██║  ██║   ██║   ╚██████╔╝██║  ██║
   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝

";

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
        Some(self.source.deref())
    }
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
struct UserExists;

impl Display for UserExists {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UserExists error - username is taken, please select a different username or login with username")
    }
}

impl Error for UserExists {}


mod term_user {
    use std::env;
    use std::io::{stdin, stdout, Write};
    use bcrypt::DEFAULT_COST;
    use regex::Regex;
    use rusqlite::Connection;
    use crate::{UserError, UserExists};

    pub struct User {
        pub username: String,
        pub password: String,
    }

    impl User {
        pub fn new() -> Result<Self, UserError> {
            let mut db_path = env::current_dir().expect("Unable to get cwd");
            db_path.pop();
            db_path = db_path.join("assets/terminator.db");
            let conn: Connection = Connection::open(db_path).expect("Unable to open db...");
            let username = Self::create_username(None, &conn);
            let not_hashed_pw = Self::create_password(None);
            let password = bcrypt::hash(&not_hashed_pw, DEFAULT_COST).expect("Unable to hash pw.");
            match username {
                Ok(un) => {
                    match conn.execute(
                        "INSERT INTO Users VALUES (?1, ?2)",
                        [&un, &password],
                    ) {
                        Ok(_) => Ok(User {username: un, password}),
                        Err(e) => Err(UserError {source: Box::new(e)}),
                    }
                },
                Err(e) => Err(e),
            }
        }

        pub fn get_user(username: &str, password: &str, conn: &Connection) -> Option<User> {
            // TODO:
        }
        fn create_username(username: Option<String>,conn: &Connection) -> Result<String, UserError> {
            let temp: String = match username {
                Some(name) => name,
                None => Self::input("Enter your desired username: "),
            };
            match conn.query_row(
                "SELECT COUNT(*) FROM Users WHERE Username = ?1",
                [&temp],
                |row| row.get(0),
            ) {
                Ok(num) if num == 0 => Ok(temp),
                Ok(_) => Err(UserError {source: Box::new(UserExists)}),
                Err(e) => Err(UserError {source: Box::new(e)}),
            }
        }

        fn create_password(password: Option<String>) -> String {
            let prompt: &str = "Password must be at least 10 characters long, have at least one \
            uppercase character, at least one number, and at least one special character (!@#$%^&*)\n\
            Please enter your desired password: ";
            let result: String;
            let temp: String = match password {
                Some(pw) => pw,
                None => Self::input(prompt),
            };
            if Self::is_valid_password(&temp) {
                result = temp;
            } else {
                loop {
                    let temp = Self::input(prompt);
                    if Self::is_valid_password(&temp) {
                        result = temp;
                        break;
                    }
                }
            }
            result
        }

        fn input(prompt: &str) -> String {
            let mut result: String = String::new();
            loop {
                result.clear();
                print!("{}", prompt);
                stdout().flush().expect("Unable to flush stdout...");
                stdin().read_line(&mut result).expect("Unable to read stdin...");
                let temp1: String = String::from(result.trim());

                if temp1.is_empty() {
                    println!("Entry must not be empty!");
                    continue;
                }

                result.clear();
                print!("Please confirm entry: ");
                stdout().flush().expect("Unable to flush stdout...");
                stdin().read_line(&mut result).expect("Unable to read stdin...");
                let temp2: &str = result.trim();

                if !temp1.eq(temp2) {
                    println!("Entries must match!");
                } else {
                    result = temp1;
                    break;
                }
            }
            result
        }

        fn is_valid_password(password: &str) -> bool {
            let uppercase_regex = Regex::new(r"[A-Z]").unwrap();
            let digit_regex = Regex::new(r"\d").unwrap();
            let special_char_regex = Regex::new(r"[!@#$%^&*]").unwrap();

            let has_uppercase = uppercase_regex.is_match(password);
            let has_digit = digit_regex.is_match(password);
            let has_spec_char = special_char_regex.is_match(password);
            let has_strong_length = password.len() >= 10;

            has_uppercase && has_digit && has_spec_char && has_strong_length
        }
    }
}

/* Original implementation of User
mod term_user {
    use bcrypt::DEFAULT_COST;
    use regex::Regex;
    use std::io::{stdin, stdout, Write};
    use crate::{BadPassword, NoSuchUser, UserError};

    pub struct User {
        pub username: String,
        pub password: String,
    }

    impl User {
        pub fn new() -> Self {
            User {
                username: String::new(),
                password: String::new(),
            }
        }

        pub fn from(username: &str, password: &str) -> Result<User, UserError> {
            if Self::is_valid_password(password) {
                if let Ok(hashed_pw) = bcrypt::hash(password, DEFAULT_COST) {
                    Ok(User {username: String::from(username), password: hashed_pw})
                } else {
                    Err(UserError {source: Box::new(NoSuchUser)})
                }
            } else {
                Err(UserError {source: Box::new(BadPassword)})
            }
        }

        pub fn get_username(&self) -> &str {
            &self.username
        }

        pub fn get_password(&self) -> &str {
            &self.password
        }

        pub fn set_password(&mut self) {
            println!("Setting a new password...");
            println!("Password must be at least 10 characters long, have at least one uppercase \
            letter, at least one number, and at least one special character (!@#$%^&*)");
            let mut new_password: String = String::new();
            loop {
                new_password.clear();
                new_password = Self::validate_input("password");
                if Self::is_valid_password(&new_password) {
                    break;
                }
            }
            self.password = new_password;
        }

        pub fn set_password_from_str(&mut self, new_password: &str) -> Result<(), UserError> {
            if Self::is_valid_password(new_password) {
                self.password = bcrypt::hash(new_password, DEFAULT_COST).expect("Unable to hash pw");
                Ok(())
            } else {
                Err(UserError {source: Box::new(BadPassword)})
            }
        }

        fn is_valid_username(username: &str) -> bool {
            !username.is_empty()
        }

        fn is_valid_password(password: &str) -> bool {
            let uppercase_regex = Regex::new(r"[A-Z]").unwrap();
            let digit_regex = Regex::new(r"\d").unwrap();
            let special_char_regex = Regex::new(r"[!@#$%^&*]").unwrap();

            let has_uppercase = uppercase_regex.is_match(password);
            let has_digit = digit_regex.is_match(password);
            let has_spec_char = special_char_regex.is_match(password);
            let has_strong_length = password.len() >= 10;

            has_uppercase && has_digit && has_spec_char && has_strong_length
        }

        fn validate_input(attrib_type: &str) -> String {
            let mut temp: String = String::new();
            let result: String;
            loop {
                temp.clear();
                print!("Enter desired {}: ", attrib_type);
                stdout().flush().expect("Unable to flush stdout...");
                stdin().read_line(&mut temp).expect("Unable to read stdin...");
                let input1 = String::from(temp.trim());

                if input1.is_empty() {
                    eprintln!("Invalid {}. Please try again.", attrib_type);
                    continue
                }

                temp.clear();
                print!("Confirm desired {}: ", attrib_type);
                stdout().flush().expect("Unable to flush stdout...");
                stdin().read_line(&mut temp).expect("Unable to read stdin...");
                let input2 = temp.trim();

                if input1.eq(input2) {
                    result = String::from(input1);
                    break;
                } else {
                    eprintln!("Entries do not match; please try again.");
                }
            }
            result
        }
    }
}
*/

/*
struct Question {
    question_id: i32,
    question_text: String,
    options: String,
    answer: String,
    explanation: String,
    exam_id: i32,
}

struct Exam {
    exam_id: i32,
    exam_title: String,
    exam_desc: String,
    questions: Vec<Question>,
}

struct Game {
    conn: Connection,
    user: User,
    exam: Exam,
}

impl Game {

    pub fn new() -> Self {
        let mut db: PathBuf = env::current_dir().expect("Unable to get current directory");
        db.pop();
        db.pop();
        db = db.join("/assets/terminator.db");
        let conn: Connection = Connection::open(db).expect("Unable to open db");

        Game {
            conn,
            user: User::new(),
        }
    }
    pub fn new_game_with_existing_user(username: &str) -> Self {


    }

    pub fn new_game_with_new_user() -> Self {

    }
}*/


fn main() {
    println!("{}", LOGO);
    // Establish the path to the database for querying whether this User exists
    let mut db: PathBuf = env::current_dir().expect("Couldn't obtain cwd.");
    db.pop();
    db = db.join("/assets/terminator.db");

    if let Ok(_conn) = Connection::open(&db) {
        println!("Successfully connected to database...");
    } else {
        eprintln!("Unable to connect to database...");
    }


    // let _user: User = select_user(&conn).unwrap();

    let src1 = NoSuchUser;
    let src2 = BadPassword;
    let err1 = UserError {source: Box::new(src1)};
    let err2 = UserError {source: Box::new(src2)};
    println!("{}", err1);
    println!("{}", err2);
}

/*
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
                let entered_name: &str = temp.trim();
                if let Ok(usr) = find_user_by_username(&conn, entered_name) {
                    user = usr;
                    break;
                } else {
                    eprintln!("Unable to get {entered_name}. Please create a new one or try again.");
                }
            },
            "n" => {
                if let Ok(usr) = create_new_user(&conn) {
                    user = usr;
                    break;
                } else {
                    eprintln!("Unable to create a new user. Please try again.");
                }
            },
            "q" => {
                println!("\n\nThank you for playing Terminator!\nGoodbye!\n\n");
                exit(0);
            },
            _ => eprintln!("Please make a valid selection!"),
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

    conn.execute(
        "INSERT INTO Users (Username, Password) VALUES (?1, ?2)",
        [&username, &password],
    ).expect("Unable to insert new user into database.");

    find_user_by_username(&conn, &username)
}

fn create_valid_password() -> String {
    let mut temp: String = String::new();
    loop {
        temp.clear();
        print!("Enter your new password: ");
        stdout().flush().expect("Unable to flush stdout...");
        let pw1 = temp.trim();

        temp.clear();
        print!("Confirm your new password: ");
        stdout().flush().expect("Unable to flush stdout...");
        let pw2 = temp.trim();

        let are_same: bool = pw1.eq(pw2);
        let has_valid_len: bool = pw1.len() < 10;
        let has_uppercase: bool;

    }
    temp
}

 */