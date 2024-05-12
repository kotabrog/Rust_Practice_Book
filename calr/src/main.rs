fn main() {
    if let Err(err) = calr::get_args().and_then(calr::run) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
