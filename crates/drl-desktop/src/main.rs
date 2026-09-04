//! Executable entry point for the native preview shell.

fn main() {
  if std::env::args().any(|argument| argument == "--validate") {
    if let Err(error) = drl_desktop::validate_demo() {
      eprintln!("drl-desktop: {error}");
      std::process::exit(2);
    }
    println!("drl-desktop demo validation: PASS");
    return;
  }

  if let Err(error) = drl_desktop::run() {
    eprintln!("drl-desktop: {error}");
    std::process::exit(1);
  }
}
