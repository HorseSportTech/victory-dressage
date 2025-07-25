fn main() {
    // Tell Cargo to rerun this build script if `.env` changes
    println!("cargo:rerun-if-changed=.env");

    // Read in Env variables
    let file = std::fs::File::open(".env").expect("Could not open .env");
    let reader = std::io::BufReader::new(file);

    for line in std::io::BufRead::lines(reader) {
        let line = line.unwrap();
        if let Some((key, value)) = line.split_once('=') {
            // Strip quotes, whitespace
            let key = key.trim();
            let value = value.trim().trim_matches('"');

            println!("cargo:rustc-env={key}={value}");
        }
    }

    // Tauri stuff
    tauri_build::build()
}
