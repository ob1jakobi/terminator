use std::env;
use std::path::PathBuf;
use rusqlite::{Connection};

const LOGO: &str = "

████████╗███████╗██████╗ ███╗   ███╗██╗███╗   ██╗ █████╗ ████████╗ ██████╗ ██████╗
╚══██╔══╝██╔════╝██╔══██╗████╗ ████║██║████╗  ██║██╔══██╗╚══██╔══╝██╔═══██╗██╔══██╗
   ██║   █████╗  ██████╔╝██╔████╔██║██║██╔██╗ ██║███████║   ██║   ██║   ██║██████╔╝
   ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║██║██║╚██╗██║██╔══██║   ██║   ██║   ██║██╔══██╗
   ██║   ███████╗██║  ██║██║ ╚═╝ ██║██║██║ ╚████║██║  ██║   ██║   ╚██████╔╝██║  ██║
   ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝

";
const ASSETS_DIR: &str = "assets";
const DB_NAME: &str = "terminator.db";




mod term_user {
    use std::ops::Deref;
    use std::error::Error;
    use std::fmt::{Display, Formatter};
    use std::io::{stdin, stdout, Write};
    use bcrypt::DEFAULT_COST;
    use regex::Regex;
    use rusqlite::{Connection, params};

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
    struct WeakPassword;

    impl Display for WeakPassword {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "Password must be at least 10 characters long, use uppercase and lowercase letters,\
        and use at least one symbol.")
        }
    }

    impl Error for WeakPassword {}

    #[derive(Debug)]
    struct PasswordMismatch;

    impl Display for PasswordMismatch {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "PasswordMismatch error - passwords must match.")
        }
    }

    impl Error for PasswordMismatch{}

    #[derive(Debug)]
    struct UserExists;

    impl Display for UserExists {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "UserExists error - username is taken, please select a different username or login with username")
        }
    }

    impl Error for UserExists {}

    #[derive(Debug)]
    struct BadQuery;

    impl Display for BadQuery {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "BadQuery - Error with query in terminator database")
        }
    }

    impl Error for BadQuery {}

    #[derive(Debug)]
    pub struct User {
        pub username: String,
        pub password: String,
    }

    impl User {
        /// Creates a new `User` and adds them to them to the database given via `conn`.
        pub fn new(conn: &Connection) -> Result<Self, UserError> {
            match Self::create_username(None, &conn) {
                Ok(username) => {
                    let password: String = Self::create_password(None);
                    Self::new_from_str(&username, &password, &conn)
                },
                Err(e) => Err(e),
            }
        }
        pub fn new_from_str(username: &str, password: &str, conn: &Connection) -> Result<Self, UserError> {
            let username = Self::create_username(Some(username.to_string()), &conn);
            let password = Self::create_password(Some(password.to_string()));
            let password = bcrypt::hash(&password, DEFAULT_COST).expect("Unable to hash pw.");
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

        /// Queries the database for a `User` with the given `username` and `password`, if they exist.
        pub fn get_user_from_str(username: &str, password: &str, conn: &Connection) -> Option<User> {
            let mut stmt = conn.prepare(
                "SELECT Username, Password FROM Users WHERE Username = ?1"
            ).expect("Unable to prepare User query statement.");

            if let Ok(user) = stmt.query_row([username], |row| {
                Ok(User {
                    username: row.get(0)?,
                    password: row.get(1)?,
                })
            }) {
                if bcrypt::verify(password, &user.password).unwrap_or(false) {
                    Some(user)
                } else {
                    None
                }
            } else {
                None
            }
        }

        pub fn change_password(&mut self, conn: &Connection) -> Result<bool, UserError> {
            let current_pw = Self::input("Enter current password: ");
            if bcrypt::verify(&current_pw, &self.password).unwrap_or(false) {
                let mut new_pw = Self::create_password(None);
                new_pw = bcrypt::hash(new_pw, DEFAULT_COST).expect("Unable to hash new password");
                match conn.execute("UPDATE Users SET Password = ? WHERE Username = ?", params![new_pw, &self.username]) {
                    Ok(_) => Ok(true),
                    Err(e) => Err(UserError {source: Box::new(e)}),
                }
            } else {
                Err(UserError {source: Box::new(PasswordMismatch)})
            }
        }


        /// Helper function that will query the database to see if a given `username` exists. If
        /// the `username` doesn't already exist, then a `Result` with the username will be returned.
        /// Otherwise, a `UserError` will be returned indicating that the desired `username` is taken.
        fn create_username(username: Option<String>, conn: &Connection) -> Result<String, UserError> {
            let temp: String = match username {
                Some(name) => name,
                None => Self::input("Enter your desired username: "),
            };
            if let Ok(_) = conn.query_row(
                "SELECT Username, Password FROM Users WHERE Username = ?1",
                [&temp],
                |row| {
                    Ok(User {
                        username: row.get(0)?,
                        password: row.get(1)?,
                    })
                }
            ) {
                Err(UserError {source: Box::new(UserExists)})
            } else {
                Ok(temp)
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



fn main() {
    println!("{}", LOGO);

    // Establish the path to the database for querying whether this User exists
    let mut db: PathBuf = env::current_dir().expect("Couldn't obtain cwd.");
    db.push(ASSETS_DIR);
    db.push(DB_NAME);

    if let Ok(conn) = Connection::open(&db) {
        println!("Successfully connected to database...");
        // TODO: Logic for playing the game
    } else {
        eprintln!("Unable to connect to database...");
    }
}