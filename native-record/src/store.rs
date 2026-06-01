use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub const STORE_VERSION: u32 = 1;
const STORE_FILE_NAME: &str = "tasks.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskStore {
    #[serde(default = "current_store_version")]
    pub version: u32,
    #[serde(default)]
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDraft {
    pub title: String,
    pub note: String,
    pub due_date: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StoreOnDisk {
    Store(TaskStore),
    Tasks(Vec<Task>),
}

fn current_store_version() -> u32 {
    STORE_VERSION
}

pub fn empty_store() -> TaskStore {
    TaskStore {
        version: STORE_VERSION,
        tasks: Vec::new(),
    }
}

pub fn default_store_path() -> PathBuf {
    dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("Record")
        .join(STORE_FILE_NAME)
}

pub fn load_or_create_store(path: &Path) -> Result<TaskStore, String> {
    ensure_parent_dir(path)?;

    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let store = empty_store();
            write_store(path, &store)?;
            return Ok(store);
        }
        Err(error) => return Err(format!("Failed to read task store: {error}")),
    };

    match parse_store(&content) {
        Ok(store) => Ok(normalize_store(store)),
        Err(_) => {
            backup_corrupt_file(path)?;
            let store = empty_store();
            write_store(path, &store)?;
            Ok(store)
        }
    }
}

pub fn save_tasks_at_path(path: &Path, tasks: Vec<Task>) -> Result<TaskStore, String> {
    validate_tasks(&tasks)?;
    let store = normalize_store(TaskStore {
        version: STORE_VERSION,
        tasks,
    });
    write_store(path, &store)?;
    Ok(store)
}

pub fn create_task(draft: &TaskDraft) -> Result<Task, String> {
    let title = draft.title.trim().to_string();
    if title.is_empty() {
        return Err("请输入任务标题".to_string());
    }

    let due_date = normalize_due_date(&draft.due_date)?;
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    Ok(Task {
        id: new_task_id(),
        title,
        note: draft.note.trim().to_string(),
        due_date,
        done: false,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn update_task(task: &Task, draft: &TaskDraft) -> Result<Task, String> {
    let title = draft.title.trim().to_string();
    if title.is_empty() {
        return Err("请输入任务标题".to_string());
    }

    let due_date = normalize_due_date(&draft.due_date)?;

    Ok(Task {
        title,
        note: draft.note.trim().to_string(),
        due_date,
        updated_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        ..task.clone()
    })
}

pub fn sort_tasks(tasks: &mut [Task]) {
    tasks.sort_by(|left, right| {
        left.done
            .cmp(&right.done)
            .then_with(|| due_date_key(left).cmp(&due_date_key(right)))
            .then_with(|| left.created_at.cmp(&right.created_at))
    });
}

pub fn is_valid_due_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }

    if !bytes
        .iter()
        .enumerate()
        .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
    {
        return false;
    }

    let year = match value[0..4].parse::<u32>() {
        Ok(year) if year > 0 => year,
        _ => return false,
    };
    let month = match value[5..7].parse::<u32>() {
        Ok(month) => month,
        _ => return false,
    };
    let day = match value[8..10].parse::<u32>() {
        Ok(day) => day,
        _ => return false,
    };

    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return false,
    };

    (1..=max_day).contains(&day)
}

fn normalize_store(mut store: TaskStore) -> TaskStore {
    store.version = STORE_VERSION;
    sort_tasks(&mut store.tasks);
    store
}

fn parse_store(content: &str) -> Result<TaskStore, serde_json::Error> {
    let store = serde_json::from_str::<StoreOnDisk>(content)?;
    Ok(match store {
        StoreOnDisk::Store(store) => store,
        StoreOnDisk::Tasks(tasks) => TaskStore {
            version: STORE_VERSION,
            tasks,
        },
    })
}

fn validate_tasks(tasks: &[Task]) -> Result<(), String> {
    for task in tasks {
        if task.id.trim().is_empty() {
            return Err("Task id cannot be empty".to_string());
        }

        if task.title.trim().is_empty() {
            return Err("Task title cannot be empty".to_string());
        }

        if let Some(due_date) = &task.due_date {
            if !is_valid_due_date(due_date) {
                return Err(format!("Invalid dueDate for task '{}'", task.id));
            }
        }
    }

    Ok(())
}

fn normalize_due_date(value: &str) -> Result<Option<String>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    if is_valid_due_date(trimmed) {
        Ok(Some(trimmed.to_string()))
    } else {
        Err("日期格式必须是 YYYY-MM-DD".to_string())
    }
}

fn due_date_key(task: &Task) -> &str {
    task.due_date.as_deref().unwrap_or("9999-12-31")
}

fn write_store(path: &Path, store: &TaskStore) -> Result<(), String> {
    ensure_parent_dir(path)?;
    let content = serde_json::to_string_pretty(store)
        .map_err(|error| format!("Failed to serialize task store: {error}"))?;
    fs::write(path, format!("{content}\n"))
        .map_err(|error| format!("Failed to write task store: {error}"))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create task store directory: {error}"))?;
    }
    Ok(())
}

fn backup_corrupt_file(path: &Path) -> Result<PathBuf, String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("tasks");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();

    for attempt in 0..100 {
        let suffix = if attempt == 0 {
            String::new()
        } else {
            format!("-{attempt}")
        };
        let backup_path = parent.join(format!("{stem}.invalid.{timestamp}{suffix}.json"));

        match fs::rename(path, &backup_path) {
            Ok(()) => return Ok(backup_path),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(format!("Failed to back up corrupt task store: {error}")),
        }
    }

    Err("Failed to choose a backup path for corrupt task store".to_string())
}

fn new_task_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("task-{nanos:x}")
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_task(id: &str, done: bool, due_date: Option<&str>, created_at: &str) -> Task {
        Task {
            id: id.to_string(),
            title: format!("Task {id}"),
            note: "A small note".to_string(),
            due_date: due_date.map(str::to_string),
            done,
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
        }
    }

    #[test]
    fn creates_empty_store_when_file_is_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(STORE_FILE_NAME);

        let store = load_or_create_store(&path).unwrap();

        assert_eq!(store, empty_store());
        assert!(path.exists());
    }

    #[test]
    fn saves_and_loads_tasks_from_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(STORE_FILE_NAME);
        let tasks = vec![
            sample_task("done", true, Some("2026-06-02"), "2026-05-31T00:00:00.000Z"),
            sample_task(
                "open",
                false,
                Some("2026-06-01"),
                "2026-05-31T00:00:00.000Z",
            ),
        ];

        save_tasks_at_path(&path, tasks).unwrap();
        let store = load_or_create_store(&path).unwrap();

        assert_eq!(store.version, STORE_VERSION);
        assert_eq!(store.tasks[0].id, "open");
        assert_eq!(store.tasks[1].id, "done");
    }

    #[test]
    fn migrates_legacy_array_store() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(STORE_FILE_NAME);
        let legacy_tasks = vec![sample_task(
            "legacy",
            false,
            None,
            "2026-05-31T00:00:00.000Z",
        )];
        fs::write(&path, serde_json::to_string(&legacy_tasks).unwrap()).unwrap();

        let store = load_or_create_store(&path).unwrap();

        assert_eq!(store.version, STORE_VERSION);
        assert_eq!(store.tasks, legacy_tasks);
    }

    #[test]
    fn backs_up_corrupt_json_and_recreates_empty_store() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(STORE_FILE_NAME);
        fs::write(&path, "this is not json").unwrap();

        let store = load_or_create_store(&path).unwrap();
        let backups = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".invalid."))
            .count();

        assert_eq!(store, empty_store());
        assert_eq!(backups, 1);
        assert!(fs::read_to_string(&path)
            .unwrap()
            .contains("\"version\": 1"));
    }

    #[test]
    fn validates_local_yyyy_mm_dd_dates() {
        assert!(is_valid_due_date("2026-05-31"));
        assert!(is_valid_due_date("2024-02-29"));
        assert!(!is_valid_due_date("2026-2-03"));
        assert!(!is_valid_due_date("2026-02-29"));
        assert!(!is_valid_due_date("2026-13-01"));
        assert!(!is_valid_due_date("0000-01-01"));
    }

    #[test]
    fn rejects_invalid_due_date_on_save() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(STORE_FILE_NAME);
        let error = save_tasks_at_path(
            &path,
            vec![sample_task(
                "bad-date",
                false,
                Some("2026-02-29"),
                "2026-05-31T00:00:00.000Z",
            )],
        )
        .unwrap_err();

        assert!(error.contains("Invalid dueDate"));
    }

    #[test]
    fn sorts_open_tasks_first_then_due_date() {
        let mut tasks = vec![
            sample_task("done-a", true, None, "2026-05-31T00:00:03.000Z"),
            sample_task("open-no-date", false, None, "2026-05-31T00:00:01.000Z"),
            sample_task(
                "open-soon",
                false,
                Some("2026-06-01"),
                "2026-05-31T00:00:02.000Z",
            ),
            sample_task(
                "done-soon",
                true,
                Some("2026-06-01"),
                "2026-05-31T00:00:04.000Z",
            ),
        ];

        sort_tasks(&mut tasks);
        let ids = tasks
            .iter()
            .map(|task| task.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec!["open-soon", "open-no-date", "done-soon", "done-a"]
        );
    }

    #[test]
    fn creates_task_from_trimmed_draft() {
        let task = create_task(&TaskDraft {
            title: "  写周报 ".to_string(),
            note: "  今天处理 ".to_string(),
            due_date: "2026-06-01".to_string(),
        })
        .unwrap();

        assert_eq!(task.title, "写周报");
        assert_eq!(task.note, "今天处理");
        assert_eq!(task.due_date.as_deref(), Some("2026-06-01"));
        assert!(!task.done);
    }
}
