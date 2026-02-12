pub fn exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}
