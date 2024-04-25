fn main() {
    if let Err(err) = tailr::get_args().and_then(tailr::run) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
