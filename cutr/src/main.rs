fn main() {
    if let Err(err) = cutr::get_args().and_then(cutr::run) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
