use log::debug;

mod simplerotate;

fn main() -> std::io::Result<()> {
    env_logger::builder().format_timestamp_nanos().init();

    let log_directory_option = std::env::args().nth(1);
    if let Some(log_directory) = log_directory_option {
        debug!("change to log_directory = {}", log_directory);
        std::env::set_current_dir(log_directory)?;
    }

    let lock_file_name = "lock";
    let max_file_size_bytes = 1 * 1024 * 1024;
    let output_file_name = "output";
    let max_output_files = 10;

    let simple_rotate = simplerotate::SimpleRotate::new(
        lock_file_name,
        max_file_size_bytes,
        output_file_name,
        max_output_files,
    );
    simple_rotate.run()?;

    Ok(())
}
