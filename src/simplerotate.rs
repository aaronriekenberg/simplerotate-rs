use log::debug;

use std::{
    convert::TryFrom,
    fs::{File, OpenOptions},
    io::Write,
    os::unix::io::AsRawFd,
};

fn flock_exclusive(file: &File) -> std::io::Result<()> {
    let raw_fd = file.as_raw_fd();

    debug!("before libc::flock raw_fd = {}", raw_fd);
    let ret = unsafe { libc::flock(raw_fd, libc::LOCK_EX) };
    debug!("after libc::flock ret = {}", ret);

    if ret < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
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

pub struct SimpleRotate {
    lock_file_name: String,
    max_file_size_bytes: usize,
    output_file_name: String,
    rotation_info_list: Vec<RotationInfo>,
}

impl SimpleRotate {
    pub fn new(
        lock_file_name: &str,
        max_file_size_bytes: usize,
        output_file_name: &str,
        max_output_files: usize,
    ) -> Self {
        let rotation_info_list =
            SimpleRotate::rotation_info_list(output_file_name, max_output_files);
        debug!("rotation_info_list = {:?}", rotation_info_list);

        Self {
            lock_file_name: lock_file_name.to_string(),
            max_file_size_bytes,
            output_file_name: output_file_name.to_string(),
            rotation_info_list,
        }
    }

    fn rotation_info_list(output_file_name: &str, max_output_files: usize) -> Vec<RotationInfo> {
        if max_output_files <= 1 {
            return Vec::new();
        }

        let mut list = Vec::with_capacity(max_output_files - 1);

        let mut i = max_output_files - 1;

        while i > 0 {
            let from_filename = match i - 1 {
                0 => output_file_name.to_string(),
                _ => format!("{}.{}", output_file_name, i - 1),
            };
            let to_filename = match i {
                0 => output_file_name.to_string(),
                _ => format!("{}.{}", output_file_name, i),
            };

            list.push(RotationInfo {
                from_filename,
                to_filename,
            });

            i -= 1;
        }

        list
    }

    fn acquire_lock_file(&self) -> std::io::Result<File> {
        debug!("begin acquire_lock_file");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.lock_file_name)?;

        flock_exclusive(&file)?;

        debug!("end acquire_lock_file");

        Ok(file)
    }

    fn initial_output_file_size(&self) -> usize {
        let u64_size = match std::fs::metadata(&self.output_file_name) {
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

    fn open_output_file_append(&self) -> std::io::Result<File> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.output_file_name)
    }

    fn open_output_file_truncate(&self) -> std::io::Result<File> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.output_file_name)
    }

    fn rotate_files(&self) {
        debug!("rotate_files");

        for rotation_info in &self.rotation_info_list {
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

    pub fn run(&self) -> std::io::Result<()> {
        let _lock_file = self.acquire_lock_file();

        let stdin = std::io::stdin();
        let mut line = String::new();
        let mut output_file_size = self.initial_output_file_size();
        let mut output_file = self.open_output_file_append()?;
        debug!("initial output_file_size = {}", output_file_size);

        loop {
            line.clear();
            let bytes_read = stdin.read_line(&mut line)?;
            debug!("read_line bytes_read = {}", bytes_read);
            if bytes_read == 0 {
                debug!("bytes_read == 0 return from run");
                return Ok(());
            }

            output_file.write_all(line.as_bytes())?;
            output_file_size += bytes_read;
            if output_file_size >= self.max_file_size_bytes {
                std::mem::drop(output_file);
                self.rotate_files();
                output_file = self.open_output_file_truncate()?;
                output_file_size = 0;
            }
        }
    }
}
