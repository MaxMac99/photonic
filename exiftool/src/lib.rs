use std::{ops::Drop, process::Stdio};
use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use snafu::{OptionExt, ResultExt, Snafu};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, Command},
    sync::{mpsc, Mutex},
};
use tokio::io::{AsyncBufRead, Lines};
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub struct Exiftool {
    process: Mutex<Child>,
    cmd_count: Mutex<u32>,
    receiver: Mutex<Receiver<(bool, String)>>,
}

#[derive(Snafu, Debug)]
pub enum ExifError {
    #[snafu(display("Could not find tool"))]
    NoTool { source: std::io::Error, backtrace: Backtrace },
    #[snafu(display("Could not parse response: {message}"))]
    Parse { message: String, backtrace: Backtrace },
    #[snafu(display("The path {path:?} does not exists"))]
    FileNotExists { path: PathBuf, backtrace: Backtrace },
    #[snafu(display("The path {path:?} is not valid"))]
    InvalidPath { path: PathBuf, backtrace: Backtrace },
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
struct Field {
    #[serde(rename = "desc")]
    description: String,
    #[serde(rename = "val")]
    value: Value,
    #[serde(rename = "num")]
    raw: Option<Value>,
}

impl From<Field> for Metadata {
    fn from(field: Field) -> Self {
        Self {
            description: field.description,
            value: field.value,
            raw: field.raw,
            information_type: None,
            specific_location: None,
            category: None,
            format: None,
        }
    }
}

impl Exiftool {
    pub async fn new() -> Result<Self, ExifError> {
        let mut process = Command::new("exiftool")
            .args(["-stay_open", "true", "-@", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context(NoToolSnafu)?;

        let stdout = process.stdout
            .take()
            .context(ParseSnafu { message: String::from("Could not connect to exiftool") })?;
        let stdout_reader = BufReader::new(stdout).lines();
        let (transmitter, receiver) = mpsc::channel(10);
        Self::start_reading(stdout_reader, transmitter.clone(), false);

        let stderr = process.stderr
            .take()
            .context(ParseSnafu { message: String::from("Could not connect to exiftool") })?;
        let stderr_reader = BufReader::new(stderr).lines();
        Self::start_reading(stderr_reader, transmitter.clone(), true);

        let exiftool = Exiftool {
            process: Mutex::from(process),
            cmd_count: Mutex::new(0),
            receiver: Mutex::from(receiver),
        };
        exiftool.send_command(String::from("-ver")).await?;

        Ok(exiftool)
    }

    fn start_reading<K: AsyncBufRead + Unpin + Send + 'static>(mut reader: Lines<K>, transmitter: Sender<(bool, String)>, error: bool) {
        tokio::spawn(async move {
            while let Some(line) = reader.next_line().await.unwrap_or(None) {
                match transmitter.send((error, String::from(line.trim()))).await {
                    Err(_) => return,
                    _ => (),
                }
            }
        });
    }

    async fn send_command(&self, cmd: String) -> Result<String, ExifError> {
        let mut process = self.process.lock().await;
        let stdin = process.stdin
            .as_mut()
            .context(ParseSnafu { message: String::from("Could not connect to exiftool") })?;

        let mut cmd_num = self.cmd_count.lock().await;
        *cmd_num += 1;
        let cmd = format!("{}\n-echo4\n{{ready{:05}}}=${{status}}\n-execute{:05}\n", cmd, cmd_num, cmd_num);
        stdin.write_all(cmd.as_bytes())
            .await
            .map_err(|err| ParseSnafu { message: err.to_string() }.build())?;

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
            return ParseSnafu { message: err_result }.fail();
        }
        Ok(result)
    }

    pub async fn read_file<P>(&self, file: P, with_binary: bool, with_groups: bool) -> Result<HashMap<String, Metadata>, ExifError>
        where P: AsRef<Path>
    {
        let path = file.as_ref()
            .to_path_buf()
            .into_os_string()
            .into_string()
            .map_err(|_| InvalidPathSnafu { path: file.as_ref().to_path_buf() }.build())?;
        if !Path::new(&path).exists() {
            return FileNotExistsSnafu { path: PathBuf::from(path) }.fail();
        }

        // -j json
        // -l long, adds desc and unconverted value
        // -g:0:1:2:6 grouping: 0=Information Type, 1=Specific Location, 2=Category, 6=EXIF/TIFF Format
        // -struct structured output
        // -t tab, adds table for unique ids
        // -b outputs metadata in binary format
        // -D adds id field in decimal format
        // -H adds id field in hexadecimal format
        // -d reformat date fields
        let mut options = String::new();
        if with_binary {
            options.push_str("\n-b");
        }
        if with_groups {
            options.push_str("\n-g:0:1:2:6");
        }
        let cmd = format!("\n-j\n-l\n-struct{}\n{}", options, path);
        let result = self.send_command(cmd).await?;
        let res: Value = serde_json::from_str(&result)
            .map_err(|err| ParseSnafu { message: err.to_string() }.build())?;
        let fields_iterator = res.as_array()
            .unwrap()
            .first()
            .unwrap()
            .as_object()
            .unwrap()
            .iter()
            .filter(|(key, _)| *key != "SourceFile");
        let items: HashMap<String, Metadata> = if with_groups {
            fields_iterator
                .flat_map(|(group, value)| {
                    value.as_object()
                        .unwrap()
                        .iter()
                        .map(|(key, value)| (group.clone(), key, value))
                })
                .map(|(group, key, value)| (group, key.clone(), serde_json::from_value::<Field>(value.clone()).unwrap()))
                .map(|(group, key, value)| {
                    let mut meta: Metadata = value.into();
                    let groups: Vec<Option<String>> = group.split(":")
                        .map(|value| if value.trim() == "" {
                            None
                        } else {
                            Some(String::from(value))
                        })
                        .collect();
                    meta.information_type = groups.get(0).cloned().flatten();
                    meta.specific_location = groups.get(1).cloned().flatten();
                    meta.category = groups.get(2).cloned().flatten();
                    meta.format = groups.get(3).cloned().flatten();
                    (key, meta)
                })
                .collect()
        } else {
            fields_iterator
                .map(|(key, value)| (key.clone(), serde_json::from_value::<Field>(value.clone()).unwrap().into()))
                .collect()
        };

        Ok(items)
    }
}

impl Drop for Exiftool {
    fn drop(&mut self) {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut process = self.process.lock().await;
                let stdin = process.stdin.as_mut();
                if let Some(stdin) = stdin {
                    debug!("Send stop to exiftool");
                    let _ = stdin
                        .write_all(format!("-stay_open\nfalse\n").as_bytes())
                        .await;
                    let _ = process.wait();
                }
                debug!("Stopped exiftool");
            });
        });
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use itertools::Itertools;

    use crate::Exiftool;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn exif_heic() {
        let exiftool = Exiftool::new().await.unwrap();

        let target_path = std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join(PathBuf::from("test/IMG_4598.HEIC"));
        let _res = exiftool
            .read_file(target_path, false, true)
            .await
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn exif_dng() {
        let exiftool = Exiftool::new().await.unwrap();

        let target_path = std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join(PathBuf::from("test/IMG_4597.DNG"));
        let res = exiftool
            .read_file(target_path, false, true)
            .await
            .unwrap();

        for key in res.keys().sorted() {
            let value = res.get(key).unwrap();
            println!("{} ({}): {} ({:?}) ({:?}, {:?}, {:?})", key, value.description, value.value, value.raw, value.information_type, value.specific_location, value.category);
        }
    }
}
