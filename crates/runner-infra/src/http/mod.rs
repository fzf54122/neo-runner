pub async fn request(method: &str, url: &str) -> Result<u16, String> {
    let method = method.trim().to_ascii_uppercase();
    match method.as_str() {
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" => {}
        _ => return Err(format!("unsupported http method: {}", method)),
    }

    let url = url.trim();
    if let Some(code) = parse_mock_status(url) {
        return Ok(code);
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(format!("invalid http url: {}", url));
    }

    let method = reqwest::Method::from_bytes(method.as_bytes())
        .map_err(|e| format!("invalid http method: {}", e))?;
    let client = reqwest::Client::new();
    let response = client
        .request(method, url)
        .send()
        .await
        .map_err(|e| format!("http request failed: {}", e))?;

    Ok(response.status().as_u16())
}

fn parse_mock_status(url: &str) -> Option<u16> {
    let prefix = "mock://status/";
    if !url.starts_with(prefix) {
        return None;
    }
    url.trim_start_matches(prefix).parse::<u16>().ok()
}
