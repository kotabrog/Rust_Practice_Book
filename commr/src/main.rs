fn main() {
    if let Err(e) = commr::get_args().and_then(commr::run) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
