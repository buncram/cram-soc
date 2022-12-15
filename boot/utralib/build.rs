fn main() {
    // the first file in this list takes priority for the vendor extensions.
    let svd_files = vec![
        "../../build/software/soc.svd".to_string(),
        "../../build/software/core.svd".to_string(),
    ];
    let mut svd_filehandles = vec![];
    for svd_filename in svd_files {
        let svd_file_path = std::path::Path::new(&svd_filename);
        println!("cargo:rerun-if-changed={}", svd_file_path.canonicalize().unwrap().display());
        svd_filehandles.push(std::fs::File::open(svd_filename).expect("couldn't open src file"));
    }
    let mut dest_file = std::fs::File::create("src/generated.rs").expect("couldn't open dest file");
    svd2utra::generate(svd_filehandles, &mut dest_file).unwrap();
}
