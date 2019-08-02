use chrono::prelude::*;
use hypertask_engine::prelude::*;
use rand::prelude::*;
use serde_json;
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::process::Command;

const ENV_VAR_AFTER_HOOK: &str = "HYPERTASK_AFTER";
const ENV_VAR_DIR_NAME: &str = "HYPERTASK_DIR";
const ENV_VAR_SHELL: &str = "SHELL";

pub struct CliContext {}

impl GetNow for CliContext {
    fn get_now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

impl PutTask for CliContext {
    fn put_task(&mut self, task: &Task) -> Result<(), String> {
        let Id(task_id) = task.get_id();

        let hyper_cortex_dir = env::var(ENV_VAR_DIR_NAME)
            .map_err(|_| format!("environment variable {} is unset", ENV_VAR_DIR_NAME))?;

        let file_path = Path::new(&hyper_cortex_dir).join(task_id);

        let file = File::create(file_path).map_err(|_| "Unable to create file")?;
        let buf_writer = BufWriter::new(file);

        serde_json::to_writer_pretty(buf_writer, &task).map_err(|_| String::from("foo?"))?;

        if let (Ok(shell), Ok(after_cmd)) = (env::var(ENV_VAR_SHELL), env::var(ENV_VAR_AFTER_HOOK))
        {
            Command::new(shell)
                .arg("-c")
                .arg(after_cmd)
                .output()
                .map_err(|_| "Failed to execute command")?;
        }

        Ok(())
    }
}

impl GenerateId for CliContext {
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
}