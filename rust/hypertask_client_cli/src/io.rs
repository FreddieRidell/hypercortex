use crate::config::CliConfig;
use hypertask_engine::prelude::*;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::rc::Rc;

pub fn put_task(config: &CliConfig, task: &Task) -> HyperTaskResult<()> {
    let Id(task_id) = &*task.get_id();

    let file_path = config.data_dir.join(task_id);

    let file = File::create(file_path).map_err(|e| {
        HyperTaskError::new(HyperTaskErrorDomain::Task, HyperTaskErrorAction::Write)
            .with_msg(|| {
                format!(
                    "could not create file handle for task with id `{}`",
                    task_id
                )
            })
            .from(e)
    })?;
    let buf_writer = BufWriter::new(file);

    serde_json::to_writer_pretty(buf_writer, &task).map_err(|e| {
        HyperTaskError::new(HyperTaskErrorDomain::Task, HyperTaskErrorAction::Write)
            .with_msg(|| format!("could not serialize task with id `{}`", task_id))
            .from(e)
    })?;

    Ok(())
}

pub fn get_input_tasks(config: &CliConfig) -> HyperTaskResult<HashMap<Rc<Id>, Rc<Task>>> {
    let task_files_iterator = fs::read_dir(&config.data_dir).map_err(|e| {
        HyperTaskError::new(HyperTaskErrorDomain::Context, HyperTaskErrorAction::Read)
            .with_msg(|| {
                format!(
                    "folder `{:}` could not be found",
                    &config.data_dir.to_str().unwrap_or("")
                )
            })
            .from(e)
    })?;

    let mut map: HashMap<Rc<Id>, Rc<Task>> = HashMap::new();

    for task_file_path_result in task_files_iterator {
        let task_file_path = task_file_path_result.map_err(|e| {
            HyperTaskError::new(HyperTaskErrorDomain::Task, HyperTaskErrorAction::Read)
                .msg("could not open task path for reading")
                .from(e)
        })?;

        let task_file = File::open(task_file_path.path()).map_err(|e| {
            HyperTaskError::new(HyperTaskErrorDomain::Task, HyperTaskErrorAction::Read)
                .with_msg(|| format!("failed to open task `{:?}`", task_file_path))
                .from(e)
        })?;

        let task: Task = serde_json::from_reader::<std::io::BufReader<std::fs::File>, Task>(
            BufReader::new(task_file),
        )
        .map_err(|e| {
            HyperTaskError::new(HyperTaskErrorDomain::Task, HyperTaskErrorAction::Read)
                .with_msg(|| format!("failed to parse task @ `{:?}`", task_file_path))
                .from(e)
        })?;

        map.insert(task.get_id(), Rc::new(task));
    }

    Ok(map)
}