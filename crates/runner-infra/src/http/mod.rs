pub async fn request(method: &str, url: &str) -> Result<u16, String> {
    let method = method.trim().to_ascii_uppercase();
    match method.as_str() {
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" => {}
        _ => return Err(format!("unsupported http method: {}", method)),
    }

    let url = url.trim();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(format!("invalid http url: {}", url));
    }

    Ok(200)
}
