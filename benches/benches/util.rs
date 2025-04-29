// Print the result of memory usages
pub fn memory_usage_output(v: &[usize], s: &str) {
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
    const BYTEUNITS: [(Option<usize>, &str); 6] = [
        (2usize.checked_pow(10), "KB"),
        (2usize.checked_pow(20), "MB"),
        (2usize.checked_pow(30), "GB"),
        (2usize.checked_pow(40), "TB"),
        (2usize.checked_pow(50), "PB"),
        (2usize.checked_pow(60), "EB"),
    ];

    for (unit, name) in BYTEUNITS.iter().rev() {
        if unit.is_some_and(|b| b < bytes) {
            return format!("{:.2} {}", bytes as f64 / unit.unwrap() as f64, name);
        }
    }
    format!("{} Bytes", bytes)
}
