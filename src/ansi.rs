pub fn red(s:&str) -> String {
    String::from(format!("\x1b[31m{}\x1b[0m", s))
}
pub fn green(s:&str) -> String {
    String::from(format!("\x1b[32m{}\x1b[0m", s))
}
// pub fn yellow(s:&str) -> String {
//     String::from(format!("\x1b[33m{}\x1b[0m", s))
// }