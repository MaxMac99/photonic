use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    ops::Drop,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::{OptionExt, ResultExt, Whatever};
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, Command},
    sync::{
        mpsc::{self, Receiver, Sender},
        Mutex,
    },
    task::JoinHandle,
};
use tracing::{debug, info};

use crate::domain::error::{DomainResult, FileNotExistsSnafu, InvalidPathSnafu, ParseSnafu};

#[derive(Debug)]
pub struct Exiftool {
    process: Mutex<Child>,
    cmd_count: Mutex<u32>,
    receiver: Mutex<Receiver<(bool, String)>>,
    reader_tasks: Vec<JoinHandle<()>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub description: String,
    pub value: Value,
    pub raw: Option<Value>,
    pub information_type: Option<String>,
    pub specific_location: Option<String>,
    pub category: Option<String>,
    pub format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field {
    #[serde(rename = "desc")]
    pub description: String,
    #[serde(rename = "val")]
    pub value: Value,
    #[serde(rename = "num")]
    pub raw: Option<Value>,
}

impl Exiftool {
    pub async fn new() -> Result<Self, Whatever> {
        debug!("Starting exiftool process");
        let mut process = Command::new("exiftool")
            .args(["-stay_open", "true", "-@", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .whatever_context("Failed to start exiftool")?;
        debug!("Exiftool process spawned successfully");

        let stdout = process
            .stdout
            .take()
            .whatever_context("Failed to get stdout from exiftool")?;
        let stdout_reader = BufReader::new(stdout).lines();
        let (transmitter, receiver) = mpsc::channel(10);
        let stdout_handle = Self::start_reading(stdout_reader, transmitter.clone(), false);

        let stderr = process
            .stderr
            .take()
            .whatever_context("Failed to get stderr from exiftool")?;
        let stderr_reader = BufReader::new(stderr).lines();
        let stderr_handle = Self::start_reading(stderr_reader, transmitter.clone(), true);
        debug!("Exiftool stdout/stderr readers started");

        let exiftool = Exiftool {
            process: Mutex::from(process),
            cmd_count: Mutex::new(0),
            receiver: Mutex::from(receiver),
            reader_tasks: vec![stdout_handle, stderr_handle],
        };
        debug!("Sending version check command to exiftool");
        let ver_result = exiftool
            .send_command(String::from("-ver"))
            .await
            .whatever_context("Failed to send ver to exiftool")?;
        info!(version = %ver_result.trim(), "Exiftool initialized");

        Ok(exiftool)
    }

    fn start_reading<K: AsyncBufRead + Unpin + Send + 'static>(
        mut reader: Lines<K>,
        transmitter: Sender<(bool, String)>,
        error: bool,
    ) -> JoinHandle<()> {
        let stream_name = if error { "stderr" } else { "stdout" };
        debug!(stream = stream_name, "Starting exiftool reader task");
        tokio::spawn(async move {
            while let Some(line) = reader.next_line().await.unwrap_or(None) {
                if transmitter
                    .send((error, String::from(line.trim())))
                    .await
                    .is_err()
                {
                    debug!(stream = stream_name, "Exiftool reader channel closed");
                    return;
                }
            }
            debug!(stream = stream_name, "Exiftool reader stream ended");
        })
    }

    async fn send_command(&self, cmd: String) -> DomainResult<String> {
        debug!(command = %cmd, "Sending command to exiftool");
        let mut process = self.process.lock().await;
        let stdin = process.stdin.as_mut().context(ParseSnafu {
            message: String::from("Could not connect to exif"),
        })?;

        let mut cmd_num = self.cmd_count.lock().await;
        *cmd_num += 1;
        let cmd = format!(
            "{}\n-echo4\n{{ready{:05}}}=${{status}}\n-execute{:05}\n",
            cmd, cmd_num, cmd_num
        );
        stdin.write_all(cmd.as_bytes()).await.map_err(|err| {
            ParseSnafu {
                message: err.to_string(),
            }
            .build()
        })?;

        let ready = format!("{{ready{:05}}}", cmd_num);

        let mut result = String::new();
        let mut err_result = String::new();
        let mut status_code: i16 = -1;
        let mut ready_count = 0;
        while let Some((error, line)) = self.receiver.lock().await.recv().await {
            if line == ready {
                ready_count += 1;
            } else if line.starts_with(&ready) {
                let status_code_str = &line[ready.len() + 1..line.len()];
                status_code = status_code_str.parse::<i16>().unwrap();
                ready_count += 1;
            } else if error {
                err_result.push_str(&line);
            } else {
                result.push_str(&format!("{}\n", line));
            }
            if ready_count == 2 {
                break;
            }
        }

        if status_code != 0 {
            debug!(cmd_num = *cmd_num, status_code, stderr = %err_result, "Exiftool command failed");
            return ParseSnafu {
                message: format!("Err: {}\n{}", err_result, result),
            }
            .fail();
        }
        Ok(result)
    }

    /// Validate and convert a path to a string
    fn validate_path<P: AsRef<Path>>(file: &P) -> DomainResult<String> {
        let path = file
            .as_ref()
            .to_path_buf()
            .into_os_string()
            .into_string()
            .map_err(|_| {
                debug!(path = ?file.as_ref(), "Invalid path: could not convert to string");
                InvalidPathSnafu {
                    path: file.as_ref().to_path_buf(),
                }
                .build()
            })?;

        if !Path::new(&path).exists() {
            debug!(path = %path, "File does not exist");
            return FileNotExistsSnafu {
                path: PathBuf::from(path),
            }
            .fail();
        }

        Ok(path)
    }

    /// Build exiftool command options
    fn build_options(with_binary: bool, with_grouping: bool) -> String {
        let mut options = String::new();
        if with_binary {
            options.push_str("\n-b");
        }
        if with_grouping {
            // -g:0:1:2:6 grouping: 0=Information Type, 1=Specific Location, 2=Category, 6=EXIF/TIFF Format
            options.push_str("\n-g:0:1:2:6");
        }
        options
    }

    /// Execute exiftool and parse the JSON result
    async fn execute_and_parse(&self, path: &str, options: &str) -> DomainResult<Value> {
        // -j json
        // -l long, adds desc and unconverted value
        // -struct structured output
        let cmd = format!("\n-j\n-l\n-struct{}\n{}", options, path);
        let result = self.send_command(cmd).await?;

        serde_json::from_str(&result).map_err(|err| {
            ParseSnafu {
                message: err.to_string(),
            }
            .build()
        })
    }

    /// Extract the first object from the result, filtering out SourceFile
    fn extract_fields(result: &Value) -> impl Iterator<Item = (&String, &Value)> {
        result
            .as_array()
            .unwrap()
            .first()
            .unwrap()
            .as_object()
            .unwrap()
            .into_iter()
            // SourceFile is not Field conformant because it is just a string
            .filter(|(key, _)| *key != "SourceFile")
    }

    pub async fn read_file<P>(
        &self,
        file: P,
        with_binary: bool,
    ) -> DomainResult<HashMap<String, Field>>
    where
        P: AsRef<Path>,
    {
        let path = Self::validate_path(&file)?;
        let options = Self::build_options(with_binary, false);
        let result = self.execute_and_parse(&path, &options).await?;

        let items: HashMap<String, Field> = Self::extract_fields(&result)
            .map(|(key, value)| {
                (
                    key.clone(),
                    serde_json::from_value::<Field>(value.clone()).unwrap(),
                )
            })
            .collect();
        Ok(items)
    }

    pub async fn read_file_grouped<P>(
        &self,
        file: P,
        with_binary: bool,
    ) -> DomainResult<HashMap<String, Metadata>>
    where
        P: AsRef<Path>,
    {
        let path = Self::validate_path(&file)?;
        let options = Self::build_options(with_binary, true);
        let result = self.execute_and_parse(&path, &options).await?;

        let items: HashMap<String, Metadata> = Self::extract_fields(&result)
            .flat_map(|(group, value)| {
                value
                    .as_object()
                    .unwrap()
                    .into_iter()
                    .map(|(key, value)| (group.clone(), key, value))
            })
            .map(|(group, key, value)| {
                (
                    group,
                    key.clone(),
                    serde_json::from_value::<Field>(value.clone()).unwrap(),
                )
            })
            .map(|(group, key, value)| {
                let groups: Vec<Option<String>> = group
                    .split(":")
                    .map(|value| {
                        if value.trim() == "" {
                            None
                        } else {
                            Some(String::from(value))
                        }
                    })
                    .collect();
                let meta = Metadata {
                    description: value.description,
                    value: value.value,
                    raw: value.raw,
                    information_type: groups.first().cloned().flatten(),
                    specific_location: groups.get(1).cloned().flatten(),
                    category: groups.get(2).cloned().flatten(),
                    format: groups.get(3).cloned().flatten(),
                };
                (key, meta)
            })
            .collect();

        Ok(items)
    }
}

impl Drop for Exiftool {
    fn drop(&mut self) {
        // Abort reader tasks so they stop holding stdout/stderr pipe handles.
        debug!("Aborting exiftool reader tasks");
        for handle in &self.reader_tasks {
            handle.abort();
        }

        // Use get_mut() to bypass the async Mutex — safe because Drop has &mut self.
        let process = self.process.get_mut();

        // Send the graceful stop command using fully synchronous I/O.
        // We cannot use async here because on a current_thread runtime,
        // block_on would deadlock (the aborted reader tasks can't be polled
        // to release their pipe handles while we hold the only thread).
        if let Some(stdin) = process.stdin.take() {
            debug!("Sending stop command to exiftool");
            match stdin.into_owned_fd() {
                Ok(fd) => {
                    let mut file = File::from(fd);
                    let _ = file.write_all(b"-stay_open\nfalse\n");
                    let _ = file.flush();
                    // Dropping file closes the pipe, signaling EOF to exiftool
                }
                Err(_) => {
                    debug!("Failed to convert stdin to fd");
                }
            }
            debug!("Sent stop command and closed exiftool stdin");
        }

        // Give exiftool a brief moment to exit gracefully, then force-kill.
        // start_kill() is synchronous (sends SIGKILL on Unix) — no runtime needed.
        std::thread::sleep(Duration::from_millis(200));
        let _ = process.start_kill();
        debug!("Exiftool shutdown complete");
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Instant};

    use super::*;

    /// Test that Exiftool shuts down cleanly when dropped during normal runtime.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_exiftool_drop_does_not_hang() {
        let exiftool = Exiftool::new().await.expect("Failed to create exiftool");

        let start = Instant::now();
        drop(exiftool);
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_secs() < 5,
            "Exiftool drop took too long ({elapsed:?}), likely deadlocked"
        );
    }

    /// Test that Exiftool shuts down cleanly after reading a file.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_exiftool_drop_after_read_does_not_hang() {
        let exiftool = Exiftool::new().await.expect("Failed to create exiftool");

        let fixtures_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/images/IMG_4598.HEIC");
        let result = exiftool.read_file(&fixtures_dir, false).await;
        assert!(result.is_ok(), "read_file failed: {:?}", result.err());

        let start = Instant::now();
        drop(exiftool);
        let elapsed = start.elapsed();

        assert!(
            elapsed.as_secs() < 5,
            "Exiftool drop took too long ({elapsed:?}), likely deadlocked"
        );
    }
}
