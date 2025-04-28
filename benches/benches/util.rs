// Print the result of memory usages
pub fn memory_usage_output(v: Vec<usize>, s: &str) {
    let min = v.iter().min().unwrap();
    let avg = v.iter().sum::<usize>() / v.len();
    let max = v.iter().max().unwrap();
    println!(
        "{:10} [min: {}, avg: {}, max: {}]",
        s,
        format_bytes(*min),
        format_bytes(avg),
        format_bytes(*max),
    );
}

// format the bytes to human-readable strings
fn format_bytes(bytes: usize) -> String {
    const KB: usize = 2usize.pow(10);
    const MB: usize = 2usize.pow(20);
    const GB: usize = 2usize.pow(30);

    if bytes < KB {
        format!("{} Bytes", bytes)
    } else if bytes < MB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if u32::MAX as usize == usize::MAX {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes < 2usize.pow(40) {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes < 2usize.pow(50) {
        format!("{:.2} TB", bytes as f64 / 2usize.pow(40) as f64)
    } else if bytes < 2usize.pow(60) {
        format!("{:.2} PB", bytes as f64 / 2usize.pow(50) as f64)
    } else {
        format!("{:.2} EB", bytes as f64 / 2usize.pow(60) as f64)
    }
}
