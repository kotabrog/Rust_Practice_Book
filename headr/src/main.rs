fn main() {
    if let Err(err) = headr::get_args().and_then(headr::run) {
        eprintln!("Error: {}", err);
    }
}
