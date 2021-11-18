use log::debug;

use std::{
    convert::TryFrom,
    error::Error,
    fs::{File, OpenOptions},
    io::Write,
};

const MAX_FILE_SIZE_BYTES: usize = 1 * 1024 * 1024;

static OUTPUT_FILE_NAME: &str = "output";

const MAX_OUTPUT_FILES: usize = 10;

fn initial_output_file_size() -> usize {
    let u64_size = match std::fs::metadata(OUTPUT_FILE_NAME) {
        Ok(m) => m.len(),
        Err(_) => 0,
    };

    match usize::try_from(u64_size) {
        Ok(size) => size,
        Err(e) => {
            debug!(
                "initial_output_file_size: error converting from u64 to usize e = {} u64_size = {}",
                e, u64_size
            );
            0
        }
    }
}

fn open_output_file_append() -> std::io::Result<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(OUTPUT_FILE_NAME)
}

fn open_output_file_truncate() -> std::io::Result<File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(OUTPUT_FILE_NAME)
}

#[derive(Debug)]
struct RotationInfo {
    from_filename: String,
    to_filename: String,
}

impl RotationInfo {
    fn from_filename(&self) -> &String {
        &self.from_filename
    }

    fn to_filename(&self) -> &String {
        &self.to_filename
    }
}

fn rotation_info_list() -> Vec<RotationInfo> {
    if MAX_OUTPUT_FILES <= 1 {
        return Vec::new();
    }

    let mut list = Vec::with_capacity(MAX_OUTPUT_FILES - 1);

    let mut i = MAX_OUTPUT_FILES - 1;

    loop {
        if i == 0 {
            break;
        }

        let from_filename = match i - 1 {
            0 => OUTPUT_FILE_NAME.to_string(),
            _ => format!("{}.{}", OUTPUT_FILE_NAME, i - 1),
        };
        let to_filename = match i {
            0 => OUTPUT_FILE_NAME.to_string(),
            _ => format!("{}.{}", OUTPUT_FILE_NAME, i),
        };

        list.push(RotationInfo {
            from_filename,
            to_filename,
        });

        i -= 1;
    }

    list
}

fn rotate_files() {
    debug!("rotate_files");

    for rotation_info in rotation_info_list() {
        debug!("calling rename rotation_info = {:?}", rotation_info);
        match std::fs::rename(rotation_info.from_filename(), rotation_info.to_filename()) {
            Ok(()) => {
                debug!("rename success {:?}", rotation_info);
            }
            Err(e) => {
                debug!(
                    "rename failed rotation_info = {:?} e = {}",
                    rotation_info, e
                );
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().format_timestamp_nanos().init();

    let log_directory_option = std::env::args().nth(1);
    if let Some(log_directory) = log_directory_option {
        debug!("change to log_directory = {}", log_directory);
        std::env::set_current_dir(log_directory)?;
    }

    let stdin = std::io::stdin();
    let mut line = String::new();
    let mut output_file_size: usize = initial_output_file_size();
    let mut output_file: File = open_output_file_append()?;

    debug!("initial output_file_size = {}", output_file_size);

    loop {
        line.clear();
        let bytes_read = stdin.read_line(&mut line)?;
        debug!("read_line bytes_read = {}", bytes_read);

        if bytes_read == 0 {
            debug!("bytes_read == 0 return from main");
            return Ok(());
        }

        output_file.write_all(line.as_bytes())?;

        output_file_size += bytes_read;

        if output_file_size >= MAX_FILE_SIZE_BYTES {
            std::mem::drop(output_file);
            rotate_files();
            output_file = open_output_file_truncate()?;
            output_file_size = 0;
        }
    }
}
