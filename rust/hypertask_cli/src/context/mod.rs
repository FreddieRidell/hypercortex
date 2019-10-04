use chrono::prelude::*;
use hypertask_config_file_opener::{
    run_string_as_shell_command, ConfigFileGetter, ConfigFileOpener, ShellExpand,
};
use hypertask_engine::prelude::*;
use platform_dirs::{AppDirs, AppUI};
use rand::seq::IteratorRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct HooksConfig {
    pub after: Option<String>,
    pub on_edit: Option<String>,
    pub before: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CliConfig {
    data_dir: PathBuf,

    pub hooks: Option<HooksConfig>,
}

impl ShellExpand for CliConfig {
    fn shell_expand(&mut self) -> () {
        let data_dir_str: &str = self
            .data_dir
            .to_str()
            .expect("could not string from data_dir");

        let expanded_data_dir = shellexpand::tilde(data_dir_str);

        self.data_dir = PathBuf::from(expanded_data_dir.into_owned());
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        let platform_dirs::AppDirs { data_dir, .. } =
            AppDirs::new(Some("hypertask-cli"), AppUI::CommandLine).unwrap();

        return Self {
            data_dir,
            hooks: None,
        };
    }
}

pub struct CliTaskIterator {
    task_files_iterator: std::fs::ReadDir,
}

impl CliTaskIterator {
    pub fn new(data_dir: &PathBuf) -> HyperTaskResult<Self> {
        let task_files_iterator = fs::read_dir(&data_dir).map_err(|e| {
            HyperTaskError::new(HyperTaskErrorDomain::Context, HyperTaskErrorAction::Read)
                .with_msg(|| {
                    format!(
                        "folder `{:}` could not be found",
                        &data_dir.to_str().unwrap_or("")
                    )
                })
                .from(e)
        })?;

        Ok(Self {
            task_files_iterator,
        })
    }
}

impl Iterator for CliTaskIterator {
    type Item = HyperTaskResult<Task>;

    fn next(&mut self) -> Option<Self::Item> {
        self.task_files_iterator.next().map(|path| {
            path.map_err(|e| {
                HyperTaskError::new(HyperTaskErrorDomain::Task, HyperTaskErrorAction::Read)
                    .msg("could not open task path for reading")
                    .from(e)
            })
            .and_then(|file_path| {
                File::open(file_path.path())
                    .map_err(|e| {
                        HyperTaskError::new(HyperTaskErrorDomain::Task, HyperTaskErrorAction::Read)
                            .with_msg(|| format!("failed to open task `{:?}`", file_path))
                            .from(e)
                    })
                    .and_then(|file| {
                        serde_json::from_reader::<std::io::BufReader<std::fs::File>, Task>(
                            BufReader::new(file),
                        )
                        .map_err(|e| {
                            HyperTaskError::new(
                                HyperTaskErrorDomain::Task,
                                HyperTaskErrorAction::Read,
                            )
                            .with_msg(|| format!("failed to parse task @ `{:?}`", file_path))
                            .from(e)
                        })
                    })
            })
        })
    }
}

pub struct CliContext {
    config_file_getter: ConfigFileGetter<CliConfig>,
}

impl CliContext {
    pub fn new() -> HyperTaskResult<CliContext> {
        let mut config_file_opener = ConfigFileOpener::new("client.toml")?;
        let config_file_getter = config_file_opener.parse()?;

        Ok(CliContext { config_file_getter })
    }
}

impl HyperTaskEngineContext<CliTaskIterator> for CliContext {
    fn get_now(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn put_task(&mut self, task: &Task) -> HyperTaskResult<()> {
        let Id(task_id) = task.get_id();

        let file_path = self.config_file_getter.get_config().data_dir.join(task_id);

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

        //TODO fix this Option nesting
        if let Some(HooksConfig {
            on_edit: Some(on_edit_cmd),
            ..
        }) = &self.config_file_getter.get_config().hooks
        {
            let output = run_string_as_shell_command(&on_edit_cmd)?;
        }

        Ok(())
    }

    fn generate_id(&mut self) -> String {
        let mut result = String::new();

        for _ in 0..NUMBER_OF_CHARS_IN_FULL_ID {
            let random = VALID_ID_CHARS
                .chars()
                .choose(&mut thread_rng())
                .expect("Couldn't get random char");

            result.push(random);
        }

        result
    }

    fn get_task_iterator(&self) -> HyperTaskResult<CliTaskIterator> {
        CliTaskIterator::new(&self.config_file_getter.get_config().data_dir).map_err(|e| {
            HyperTaskError::new(HyperTaskErrorDomain::Context, HyperTaskErrorAction::Read)
                .msg("could not open tasks folder for reading")
                .from(e)
        })
    }

    fn finalize_mutations(&self) -> HyperTaskResult<()> {
        if let Some(hooks) = &self.config_file_getter.get_config().hooks {
            if let Some(after_cmd) = &hooks.after {
                match run_string_as_shell_command(after_cmd) {
                    Ok(output) => println!("{}", output),
                    Err(output) => println!("{}", output),
                }
            }
        };

        Ok(())
    }
}
