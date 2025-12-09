use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Print a success message with a green checkmark
pub fn success(msg: &str) {
    println!("{} {}", style("✓").green().bold(), msg);
}

/// Print an info message with a blue icon
pub fn info(msg: &str) {
    println!("{} {}", style("ℹ").blue().bold(), msg);
}

/// Print a warning message with a yellow icon
pub fn warning(msg: &str) {
    println!("{} {}", style("⚠").yellow().bold(), msg);
}

/// Print an error message with a red cross
#[allow(dead_code)]
pub fn error(msg: &str) {
    eprintln!("{} {}", style("✗").red().bold(), msg);
}

/// Print a section header
pub fn header(msg: &str) {
    println!("\n{}", style(msg).bold().underlined());
}

/// Print a list item
pub fn list_item(key: &str, value: &str) {
    println!("  {} {}", style(key).dim(), value);
}

/// Print a highlighted value
pub fn highlight(msg: &str) {
    println!("{}", style(msg).cyan().bold());
}

/// Print a dim/secondary message
pub fn dim(msg: &str) {
    println!("{}", style(msg).dim());
}

/// Print a tip message
pub fn tip(msg: &str) {
    println!("\n{} {}", style("💡").bold(), style(msg).italic());
}

/// Print a plain message without icons
pub fn plain(msg: &str) {
    println!("{}", msg);
}

/// Print a bullet list item
pub fn bullet(msg: &str) {
    println!("  • {}", msg);
}

/// Print an empty line
pub fn newline() {
    println!();
}

/// Create a spinner with elapsed time for all operations
pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg} {elapsed_precise:.dim}")
            .unwrap()
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Finish spinner with success message
pub fn finish_spinner_success(pb: ProgressBar, msg: &str) {
    pb.finish_with_message(format!("{} {}", style("✓").green().bold(), msg));
}

/// Finish spinner with error message
pub fn finish_spinner_error(pb: ProgressBar, msg: &str) {
    pb.finish_with_message(format!("{} {}", style("✗").red().bold(), msg));
}

/// Finish spinner with warning message
pub fn finish_spinner_warning(pb: ProgressBar, msg: &str) {
    pb.finish_with_message(format!("{} {}", style("⚠").yellow().bold(), msg));
}
