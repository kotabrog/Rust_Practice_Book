fn main() {
    if let Err(err) = fortuner::get_args().and_then(fortuner::run) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
