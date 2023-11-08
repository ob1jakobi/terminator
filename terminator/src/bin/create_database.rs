use bcrypt::DEFAULT_COST;
use chrono::{DateTime, Datelike, Utc};
use rusqlite::Connection;
use std::{env, fs};
use std::fs::{create_dir, read_to_string};
use std::io::{ErrorKind, stdin, stdout, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

const ASSET_DIR_NAME: &str = "assets";
const DATABASE_NAME: &str = "terminator.db";
const USERS_TABLE: &str = "users.sql";
const EXAMS_TABLE: &str = "exams.sql";
const EXAM_CREATION_TABLE: &str = "exam_creation.sql";
const QUESTIONS_TABLE: &str = "questions.sql";

#[doc(hidden)]
/// Creates the database, `terminator.db`, and the tables for `terminator.db`.
/// The schemas for each of the tables are as follows:
///
/// - Users (**Username**: Text, Password: Text)
/// - Exams (**ExamID**: Int, Title: Text, Description: Text)
/// - ExamCreation (_**ExamID**_: Int, _**CreatorUsername**_: Text, DateCreated: Text)
/// - Questions (**QuestionID**: Int, QuestionText: Text, Options: Text, CorrectAnswer: Text, Explanation: Text, *ExamID*: Int)
/// - UserQuestionResponses (ResponseID, )
fn create_database_and_tables(db_path: &Path) -> rusqlite::Result<()> {
    // Creates the database if it doesn't exists, and opens it for updating.
    let conn = Connection::open(db_path)?;

    // TODO: Delete the drop statements and the code to execute it when done testing
    println!("Dropping tables...");
    let drop_usr_q_res = "DROP TABLE IF EXISTS UserQuestionResponses";
    let drop_questions = "DROP TABLE IF EXISTS Questions";
    let drop_exam_cre = "DROP TABLE IF EXISTS ExamCreation";
    let drop_exams = "DROP TABLE IF EXISTS Exams";
    let drop_users = "DROP TABLE IF EXISTS Users";

    conn.execute(drop_usr_q_res, [])?;
    conn.execute(drop_questions, [])?;
    conn.execute(drop_exam_cre, [])?;
    conn.execute(drop_exams, [])?;
    conn.execute(drop_users, [])?;
    // TODO: End of delete for testing

    let users_sql =
        "CREATE TABLE IF NOT EXISTS Users (
           Username TEXT NOT NULL UNIQUE,
           Password TEXT NOT NULL
        )";
    conn.execute(users_sql, [])?;

    let exams_sql =
    "CREATE  TABLE IF NOT EXISTS Exams (
       ExamID INTEGER PRIMARY KEY AUTOINCREMENT,
       Title TEXT NOT NULL,
       Description TEXT
    )";
    conn.execute(exams_sql, [])?;

    let exam_creat_sql =
    "CREATE TABLE IF NOT EXISTS ExamCreation (
       ExamID INTEGER NOT NULL,
       CreatorUsername TEXT NOT NULL,
       DateCreated TEXT NOT NULL,
       PRIMARY KEY (ExamID, CreatorUsername),
       FOREIGN KEY (ExamID) REFERENCES Exams(ExamID),
       FOREIGN KEY (CreatorUsername) REFERENCES Users(Username)
    )";
    conn.execute(exam_creat_sql, [])?;

    let questions_sql =
    "CREATE TABLE IF NOT EXISTS Questions (
       QuestionID INTEGER PRIMARY KEY AUTOINCREMENT,
       ExamID INTEGER NOT NULL,
       QuestionText TEXT NOT NULL,
       Options TEXT NOT NULL,
       CorrectAnswer TEXT NOT NULL,
       Explanation TEXT NOT NULL,
       FOREIGN KEY (ExamID) REFERENCES Exams(ExamID)
    )";
    conn.execute(questions_sql, [])?;

    let user_question_responses_sql =
    "CREATE TABLE IF NOT EXISTS UserQuestionResponses (
       ResponseID INTEGER PRIMARY KEY AUTOINCREMENT,
       Username TEXT NOT NULL,
       QuestionID INTEGER NOT NULL,
       IsCorrect INTEGER DEFAULT 0,
       Timestamp TEXT,
       FOREIGN KEY (Username) REFERENCES Users (Username),
       FOREIGN KEY (QuestionID) REFERENCES Questions (QuestionID)
    )";
    conn.execute(user_question_responses_sql, [])?;

    Ok(())
}

#[doc(hidden)]
/// Executes SQL commands from a `.sql` file for performing functions on tables within the
/// database passed via `db_path`.
///
/// # Arguments
/// * `db_path` - a reference to the `Path` that leads to the database (e.g., `.db` file).
/// * `sql_file_path` - a reference to the `PathBuf` that leads to the SQL file for adjusting some
/// table within the database located at `db_path`.
///
/// **Note: the `Exams` and `Users` tables must have their commands executed before calling an
/// `ExamCreation` adjustment since the `ExamCreation` table requires information from both the
/// `Users` table (for the `CreatorUsername`) and the `Exams` table (for the `ExamID`) foreign
/// keys.**
fn execute_sql_from_file(db_path: &Path, sql_file_path: &PathBuf) {
    // Open the database connection
    let conn: Connection = Connection::open(db_path).expect("Unable to open connection to db.");

    if let Ok(sql) = read_to_string(sql_file_path) {
        match conn.execute_batch(&sql) {
            Ok(_) => println!("Batch sql execution successful."),
            Err(e) => eprintln!("Unable to execute batch sql: {}", e),
        }
    } else {
        eprintln!("Couldn't convert sql batch script path to string...");
    }
}


fn main() {
    let mut base_path: PathBuf = env::current_dir().expect("Unable to get current directory...");
    base_path.pop();
    base_path.pop();
    base_path.push(ASSET_DIR_NAME);

    match create_dir(&base_path) {
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            println!("The {} directory already exists; no need to create it...", ASSET_DIR_NAME);
        },
        Err(e) => {
            eprintln!("Error occurred and caught by me: {}", e);
            exit(1);
        },
        Ok(()) => println!("Created {} directory.", ASSET_DIR_NAME),

    }

    // Database path
    let mut db_path: PathBuf = PathBuf::from(&base_path);
    db_path = db_path.join(DATABASE_NAME);
    println!("db_path:\t{:?}", db_path);

    /*
    // Establish paths to DB and .sql files for each DB Table
    // Database path
    let mut db_path: PathBuf = PathBuf::from(&base_path);
    db_path.push(DATABASE_NAME);

    // Users table path
    let mut users_path: PathBuf = PathBuf::from(&base_path);
    users_path.push(USERS_TABLE);

    // Exams table path
    let mut exams_path: PathBuf = PathBuf::from(&base_path);
    exams_path.push(EXAMS_TABLE);

    // ExamCreation table path
    let mut exam_creation_path: PathBuf = PathBuf::from(&base_path);
    exam_creation_path.push(EXAM_CREATION_TABLE);

    // Questions table path
    let mut questions_path: PathBuf = PathBuf::from(&base_path);
    questions_path.push(QUESTIONS_TABLE);
     */

    match create_database_and_tables(&db_path) {
        Err(e) => {
            eprintln!("An error occurred setting up the database/tables: {}", e);
            exit(1);
        },
        _ => println!("Database and tables created successfully..."),
    }



    let mut test_sql_batch = PathBuf::from(&base_path);
    test_sql_batch = test_sql_batch.join("test_script.sql");

    execute_sql_from_file(&db_path, &test_sql_batch);

    //let DATABASE: PathBuf = PathBuf::from(format!("{}{}", "./", DATABASE_NAME));

    /*
    create_database_tables(&test).expect("Unable to run create_database_tables()...");
    get_and_insert_user(&test).expect("Unable to run get_and_insert_user");
     */

    // let DATABASE_PATH: PathBuf = PathBuf::from("../../assets/");
    // create_database_tables()?;
    // execute_sql_from_file(&DATABASE_PATH, format!("{}{}", SQL_FILE_PATH, "TBD.sql"))?;
    // insert_username()?;
    // execute_sql_from_file(DATABASE_PATH, "../../exam_creation.sql")?;
    // execute_sql_from_file(DATABASE_PATH, "../../questions.sql")?;
}


/*
fn _add_exam() {
    let db_path: PathBuf = PathBuf::from(format!(
        "..{}..{}{}",
        MAIN_SEPARATOR_STR, MAIN_SEPARATOR_STR, ASSET_DIR_NAME
    ));
    let _conn: Connection = Connection::open(db_path).expect("Unable to access DB for adding exam");

    fn _add_question_to_exam(_exam: i32) -> bool {
        let mut result: bool = false;
        // TODO:
        return result;
    }

    fn _add_exam_manually() -> bool {
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

    fn _add_exam_by_script() -> bool {
        let mut result: bool = false;
        // TODO
        result
    }

    fn _add_exam_creator(exam_id: i32, creator_username: String, conn: &Connection) -> bool {
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
                    1 => _add_exam_by_script(),
                    _ => _add_exam_manually(),
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
 */


/*
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
 */