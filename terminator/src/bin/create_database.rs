use bcrypt::DEFAULT_COST;
use chrono::{DateTime, Datelike, Utc};
use rusqlite::Connection;
use std::env;
use std::io::{stdin, stdout, Write};
use std::path::{Path, PathBuf, MAIN_SEPARATOR_STR};

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
        Username TEXT NOT NULL UNIQUE,\
        Password TEXT,\
        )",
        [],
    )?;

    // Create Exams Table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS Exams (\
        ExamID INTEGER PRIMARY KEY AUTOINCREMENT,\
        Title TEXT NOT NULL,\
        Description TEXT,\
        )",
        [],
    )?;

    // Create the Exam Creation Table; DateCreated is YYYY-MM-DD format
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ExamCreation (\
        ExamID INTEGER NOT NULL,\
        CreatorUsername TEXT NOT NULL,\
        DateCreated TEXT NOT NULL,\
        PRIMARY KEY (ExamID, CreatorUsername)
        FOREIGN KEY (ExamID) REFERENCES Exams(ExamID),\
        FOREIGN KEY (CreatorUsername) REFERENCES Users(Username)\
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

fn add_exam() {
    let db_path: PathBuf = PathBuf::from(format!(
        "..{}..{}{}",
        MAIN_SEPARATOR_STR, MAIN_SEPARATOR_STR, ASSET_DIR_NAME
    ));
    let conn: Connection = Connection::open(db_path).expect("Unable to access DB for adding exam");

    fn add_question_to_exam(exam: i32) -> bool {
        let mut result: bool = false;
        // TODO:
        return result;
    }

    fn add_exam_manually() -> bool {
        let mut result: bool = false;
        let mut input_string: String = String::new();

        input_string.clear();
        print!("Enter the ExamID: ");
        stdout().flush().expect("unable to flush stdout");
        stdin()
            .read_line(&mut input_string)
            .expect("unable to read stdin...");

        if let Ok(exam_id) = input_string.trim().parse::<i32>() {
            input_string.clear();
            print!("Enter the exam creator's username: ");
            stdout().flush().expect("Unable to flush stdout");
            stdin()
                .read_line(&mut input_string)
                .expect("unable to read stdin...");

            if let Ok(creator) = input_string.trim().parse::<String>() {
                input_string.clear();
                println!("Is the following entry correct?");
                println!("\tExam Id:\t{}, Creator:\t{}", exam_id, creator);
                print!("Is the above correct (Y/n)?:");
                stdin()
                    .read_line(&mut input_string)
                    .expect("Unable to read stdin...");
            } else {
                println!("Please enter a valid choice...");
            }
        } else {
            println!("Please enter a valid choice...");
        }
        result
    }

    fn add_exam_by_script() -> bool {
        let mut result: bool = false;
        // TODO
        result
    }

    fn add_exam_creator(exam_id: i32, creator_username: String, conn: &Connection) -> bool {
        let mut result: bool = false;
        let current_date: DateTime<Utc> = chrono::Utc::now();
        let year = current_date.year();
        let month = current_date.month();
        let day = current_date.day();

        if conn
            .execute(
                "INSERT INTO\
            ExamCreation (ExamID, CreatorUsername, DateCreated) \
            VALUES (?1, ?2, ?3)",
                [
                    &exam_id,
                    &creator_username,
                    format!("{}-{}-{}", year, month, day).as_str(),
                ],
            )
            .is_err()
        {
            println!("Unable to update ExamCreator Table");
        } else {
            result = true;
        }

        result
    }

    let mut input_string: String = String::new();

    // Use input to see if the user wants to enter a .sql script or manually enter
    loop {
        input_string.clear();
        print!("Enter '1' to enter exam via .sql script or '2' to enter manually: ");
        stdout().flush().expect("Unable to flush stdout...");
        stdin()
            .read_line(&mut input_string)
            .expect("Unable to parse stdin...");

        let choice = input_string.trim().parse::<i32>();
        match choice {
            Ok(num) if num == 1 || num == 2 => {
                let completed: bool = match num {
                    1 => add_exam_by_script(),
                    _ => add_exam_manually(),
                };
                if completed {
                    break;
                } else {
                    println!("Please enter a valid choice...");
                }
            }
            _ => println!("Please enter a valid choice..."),
        }
    }
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

    conn.execute(
        "INSERT INTO Users (Username, Password, Email) VALUES (?1, ?2)",
        [&username, &password],
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
