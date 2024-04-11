pub fn bytes_to_readable_string(bytes: u64) -> String {
    let kilobytes = 1024.0;
    let megabytes = kilobytes * 1024.0;
    let gigabytes = megabytes * 1024.0;
    let terabytes = gigabytes * 1024.0;

    if bytes as f64 >= terabytes {
        format!("{:.2} TB", bytes as f64 / terabytes)
    } else if bytes as f64 >= gigabytes {
        format!("{:.2} GB", bytes as f64 / gigabytes)
    } else if bytes as f64 >= megabytes {
        format!("{:.2} MB", bytes as f64 / megabytes)
    } else if bytes as f64 >= kilobytes {
        format!("{:.2} KB", bytes as f64 / kilobytes)
    } else {
        format!("{} bytes", bytes)
    }
}