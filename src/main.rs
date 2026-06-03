use gitronics::run_cli;

fn main() {
    if let Err(e) = run_cli(std::env::args()) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
