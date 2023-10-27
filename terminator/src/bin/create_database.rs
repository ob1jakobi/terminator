use bcrypt::DEFAULT_COST;
use rusqlite::Connection;
use std::env;
use std::io::{stdin, stdout, Write};
use std::path::{Path, PathBuf};

const ASSET_DIR_NAME: &str = "assets";
const DATABASE_NAME: &str = "terminator.db";
const USERS_TABLE: &str = "users.sql";
const EXAMS_TABLE: &str = "exams.sql";
const EXAM_CREATION_TABLE: &str = "exam_creation.sql";
const QUESTIONS_TABLE: &str = "questions.sql";

fn create_database_tables(db_path: &Path) -> rusqlite::Result<()> {
    // Creates the database if it doesn't exists, and opens it for updating.
    let conn = Connection::open(db_path)?;

    // Create the Users Table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS Users (\
        UserID INTEGER PRIMARY KEY AUTOINCREMENT,\
        Username TEXT NOT NULL UNIQUE,\
        Password TEXT,\
        Email TEXT\
        )",
        [],
    )?;

    // Create Exams Table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS Exams (\
        ExamID INTEGER PRIMARY KEY AUTOINCREMENT,\
        Title TEXT,\
        Description TEXT,\
        Duration INTEGER\
        )",
        [],
    )?;

    // Create the Exam Creation Table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ExamCreation (\
        ExamID INTEGER PRIMARY KEY,\
        CreatorUserID INTEGER,\
        DateCreateD TEXT,\
        FOREIGN KEY (ExamID) REFERENCES Exams(ExamID),\
        FOREIGN KEY (CreatorUserID) REFERENCES Users(UserID)\
        )",
        [],
    )?;

    // Create the Questions Table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS Questions (\
        QuestionID INTEGER PRIMARY KEY AUTOINCREMENT,\
        QuestionText TEXT,\
        Options TEXT,\
        CorrectAnswer TEXT,\
        Explanation TEXT,\
        ExamID INTEGER,\
        FOREIGN KEY (ExamID) REFERENCES Exams(ExamID)\
        )",
        [],
    )?;

    Ok(())
}

fn _execute_sql_from_file(db_path: &Path, sql_file_path: &Path) {
    // Open the database connection
    let conn: Connection = Connection::open(db_path).expect("Unable to open connection to db.");

    // Read and execute the SQL from the .sql file
    let sql: String =
        std::fs::read_to_string(sql_file_path).expect("Create string from sql file path...");

    conn.execute_batch(&sql)
        .expect("Unable to execute sql batch code");
}

fn get_and_insert_user(db_path: &Path) -> rusqlite::Result<()> {
    fn get_valid_username(c: &Connection) -> String {
        let mut temp: String = String::new();
        let username: String;
        loop {
            temp.clear();
            print!("Enter your username: ");
            stdout().flush().expect("Unable to flush stdout...");

            stdin()
                .read_line(&mut temp)
                .expect("Couldn't read stdin...");
            let name = String::from(temp.trim());

            if !name.is_empty() {
                let username_exists: i32 = c
                    .query_row(
                        "SELECT COUNT(*) FROM Users WHERE Username = ?1",
                        [&name],
                        |row| row.get(0),
                    )
                    .expect("Unable to identify if username already exists by count");

                if username_exists == 0 {
                    username = name;
                    break;
                } else {
                    println!("Username already exists; please enter a new one...");
                }
            } else {
                println!("Please enter a valid username...");
            }
        }
        username
    }

    fn get_valid_input(input_type: &str) -> String {
        let mut temp: String = String::new();
        let validated_input: String;
        loop {
            temp.clear();
            print!("Enter your {}: ", input_type);
            stdout().flush().expect("Unable to flush stdout...");
            stdin()
                .read_line(&mut temp)
                .expect("Unable to read stdin...");
            let vi1 = String::from(temp.trim());

            temp.clear();
            print!("Enter your {} again: ", input_type);
            stdout().flush().expect("Unable to flush stdout...");
            stdin()
                .read_line(&mut temp)
                .expect("Unable to read stdin...");
            let vi2 = String::from(temp.trim());

            if vi1.eq(&vi2) {
                validated_input = vi2;
                break;
            } else {
                println!("{}s do not match, please try again...", input_type);
            }
        }
        if input_type.eq_ignore_ascii_case("password") {
            bcrypt::hash(validated_input, DEFAULT_COST).unwrap()
        } else {
            validated_input
        }
    }

    let conn: Connection = Connection::open(db_path)?;

    let username: String = get_valid_username(&conn);
    let password: String = get_valid_input("Password");
    let email: String = get_valid_input("Email");

    conn.execute(
        "INSERT INTO Users (Username, Password, Email) VALUES (?1, ?2, ?3)",
        [&username, &password, &email],
    )?;

    Ok(())
}
fn main() {
    let mut base_path: PathBuf = env::current_dir().expect("Unable to get current directory...");
    base_path.pop();
    base_path.pop();
    base_path.push(ASSET_DIR_NAME);

    // Establish paths to DB and .sql files for each DB Table
    let mut db_path: PathBuf = PathBuf::from(&base_path);
    db_path.push(DATABASE_NAME);
    let mut users_path: PathBuf = PathBuf::from(&base_path);
    users_path.push(USERS_TABLE);
    let mut exams_path: PathBuf = PathBuf::from(&base_path);
    exams_path.push(EXAMS_TABLE);
    let mut exam_creation_path: PathBuf = PathBuf::from(&base_path);
    exam_creation_path.push(EXAM_CREATION_TABLE);
    let mut questions_path: PathBuf = PathBuf::from(&base_path);
    questions_path.push(QUESTIONS_TABLE);

    let test = PathBuf::from(format!("{}{}", "./", DATABASE_NAME));

    create_database_tables(&test).expect("Unable to run create_database_tables()...");
    get_and_insert_user(&test).expect("Unable to run get_and_insert_user");

    // let DATABASE_PATH: PathBuf = PathBuf::from("../../assets/");
    // create_database_tables()?;
    // execute_sql_from_file(&DATABASE_PATH, format!("{}{}", SQL_FILE_PATH, "TBD.sql"))?;
    // insert_username()?;
    // execute_sql_from_file(DATABASE_PATH, "../../exam_creation.sql")?;
    // execute_sql_from_file(DATABASE_PATH, "../../questions.sql")?;
}
