pub fn error(line: &u32, message: &str) {
    report(line, "", message);
}

pub fn report(line: &u32, location: &str, message: &str) {
    eprintln!("[line {}] Error{}: {}", line, location, message);
}