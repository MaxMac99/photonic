use std::{ops::Drop, path::PathBuf, process::Stdio};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;

use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Debug)]
pub enum ExifError {
    CouldNotFindToolError(std::io::Error),
    ParseError(String),
    InvalidPathError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub description: String,
    pub value: Value,
    pub raw: Option<Value>,
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
            .map_err(|err| ExifError::CouldNotFindToolError(err))?;

        let stdout = process.stdout
            .take()
            .ok_or(ExifError::ParseError(String::from("Could not connect to exiftool")))?;
        let stdout_reader = BufReader::new(stdout).lines();
        let (transmitter, receiver) = mpsc::channel(10);
        Self::start_reading(stdout_reader, transmitter.clone(), false);

        let stderr = process.stderr
            .take()
            .ok_or(ExifError::ParseError(String::from("Could not connect to exiftool")))?;
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
            .ok_or(ExifError::ParseError(String::from("Could not connect to exiftool")))?;

        let mut cmd_num = self.cmd_count.lock().await;
        *cmd_num += 1;
        let cmd = format!("{}\n-echo4\n{{ready{:05}}}=${{status}}\n-execute{:05}\n", cmd, cmd_num, cmd_num);
        stdin.write_all(cmd.as_bytes())
            .await
            .map_err(|err| ExifError::ParseError(err.to_string()))?;

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
            return Err(ExifError::ParseError(err_result));
        }
        Ok(result)
    }

    pub async fn read_file(&self, file: PathBuf) -> Result<HashMap<String, Metadata>, ExifError> {
        let path = file
            .into_os_string()
            .into_string()
            .map_err(|_| ExifError::InvalidPathError)?;
        if !Path::new(&path).exists() {
            return Err(ExifError::InvalidPathError);
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
        let cmd = format!("\n-j\n-l\n-struct\n-b\n{}", path);
        let result = self.send_command(cmd).await?;
        let res: Value = serde_json::from_str(&result)
            .map_err(|err| ExifError::ParseError(err.to_string()))?;
        let items: HashMap<String, Metadata> = res.as_array()
            .unwrap()
            .first()
            .unwrap()
            .as_object()
            .unwrap()
            .iter()
            .filter(|(key, _)| key.deref() != "SourceFile")
            .map(|(key, value)| (key.clone(), serde_json::from_value::<Field>(value.clone()).unwrap().into()))
            .collect();

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
            .read_file(target_path)
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
        let _res = exiftool
            .read_file(target_path)
            .await
            .unwrap();
    }
}
