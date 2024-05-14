fn main() {
    if let Err(err) = lsr::get_args().and_then(lsr::run) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
