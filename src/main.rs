use log::debug;

mod simplerotate;

fn main() -> std::io::Result<()> {
    env_logger::builder().format_timestamp_nanos().init();

    let log_directory_option = std::env::args().nth(1);
    if let Some(log_directory) = log_directory_option {
        debug!("change to log_directory = {}", log_directory);
        std::env::set_current_dir(log_directory)?;
    }

    let simple_rotate = simplerotate::SimpleRotateBuilder::new()
        .lock_file_name("lock")
        .max_file_size_bytes(1 * 1024 * 1024)
        .output_file_name("output")
        .max_output_files(10)
        .build();

    simple_rotate.run()?;

    Ok(())
}
