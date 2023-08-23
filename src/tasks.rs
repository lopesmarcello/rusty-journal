use chrono::{serde::ts_seconds, DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Task {
    pub text: String,

    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl Task {
    pub fn new(text: String) -> Task {
        let created_at: DateTime<Utc> = Utc::now();
        Task { text, created_at }
    }
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let created_at = self.created_at.with_timezone(&Local).format("%F %H:%M");
        write!(f, "{:<50} [{}]", self.text, created_at)
    }
}

fn collect_tasks(mut file: &File) -> Result<Vec<Task>> {
    file.seek(SeekFrom::Start(0))?; // Rewind file
    let tasks = match serde_json::from_reader(file) {
        Ok(tasks) => tasks,
        Err(e) if e.is_eof() => Vec::new(),
        Err(e) => Err(e)?,
    };
    file.seek(SeekFrom::Start(0))?; // Rewind the file after
    Ok(tasks)
}

pub fn add_task(journal_path: PathBuf, task: Task) -> Result<()> {
    // Open the file
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(journal_path)?;

    // Consume the file's contents as a vector of tasks
    let mut tasks = collect_tasks(&file)?;

    // Write the modified task list back into the file
    tasks.push(task);
    serde_json::to_writer(file, &tasks)?;

    Ok(())
}

pub fn complete_task(journal_path: PathBuf, task_position: usize) -> Result<()> {
    // Open the file
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(journal_path)?;

    // Consume the file's content as a vector of tasks
    let mut tasks = collect_tasks(&file)?;

    // Remove the task
    if task_position == 0 || task_position > tasks.len() {
        return Err(Error::new(ErrorKind::InvalidInput, "Invalid Task ID"));
    }
    tasks.remove(task_position - 1);

    // Rewind and truncate the file
    file.set_len(0)?;

    // Write the modified task list back into the file
    serde_json::to_writer(file, &tasks)?;
    Ok(())
}

pub fn list_tasks(journal_path: PathBuf) -> Result<()> {
    let file = OpenOptions::new().read(true).open(journal_path)?;
    let tasks = collect_tasks(&file)?;

    if tasks.is_empty() {
        println!("Task list is empty!")
    } else {
        let mut order: u32 = 1;
        for task in tasks {
            println!("{}: {}", order, task);
            order += 1;
        }
    }

    Ok(())
}
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn initialize_new_file_empty() {
        if fs::read(PathBuf::from("./test1.json")).is_ok() {
            fs::remove_file(PathBuf::from("./test1.json")).expect("Couldn't clean test file up");
        };

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("./test1.json")
            .expect("Couldn't reade file");

        let tasks = collect_tasks(&file).expect("Couldn't read file");

        let empty_vec: Vec<Task> = vec![];
        assert_eq!(empty_vec, tasks);
        fs::remove_file(PathBuf::from("./test1.json")).expect("Couldn't clean test file up")
    }

    #[test]
    fn create_new_todo_and_save_to_file() {
        if fs::read(PathBuf::from("./test2.json")).is_ok() {
            fs::remove_file(PathBuf::from("./test2.json")).expect("Couldn't clean test file up");
        };
        let task = Task::new(String::from("Teste"));

        add_task(PathBuf::from("./test2.json"), task.clone()).expect("Couldn't add task");

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(PathBuf::from("./test2.json"))
            .expect("Couldn't reade file");

        let tasks = collect_tasks(&file).expect("Couldn't read file");

        assert_eq!(task.text, tasks[0].text);
        fs::remove_file(PathBuf::from("./test2.json")).expect("Couldn't clean test file up")
    }

    #[test]
    fn completes_item_and_removes_from_list() {
        if fs::read(PathBuf::from("./test3.json")).is_ok() {
            fs::remove_file(PathBuf::from("./test3.json")).expect("Couldn't clean test file up");
        };
        let task = Task::new(String::from("Teste"));

        add_task(PathBuf::from("./test3.json"), task.clone()).expect("Couldn't add task");
        add_task(PathBuf::from("./test3.json"), task.clone()).expect("Couldn't add task");
        add_task(PathBuf::from("./test3.json"), task.clone()).expect("Couldn't add task");

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(PathBuf::from("./test3.json"))
            .expect("Couldn't read file");

        let tasks = collect_tasks(&file).expect("Couldn't read file");

        assert_eq!(3, tasks.len());

        complete_task(PathBuf::from("./test3.json"), 2).expect("Couldn't complete task number 1");

        let updated_tasks = collect_tasks(&file).expect("Couldn't read file");

        assert_eq!(2, updated_tasks.len());

        fs::remove_file(PathBuf::from("./test3.json")).expect("Couldn't clean test file up")
    }
}
