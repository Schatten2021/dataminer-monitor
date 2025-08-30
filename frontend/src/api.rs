pub async fn post_request<Request: serde::Serialize, Response: for<'a> serde::Deserialize<'a>>(path: &'static str, data: &Request) -> Result<Response, gloo_net::Error> {
    #[cfg(debug_assertions)]
    const SOURCE_URL: &'static str = "http://localhost:8000";
    #[cfg(not(debug_assertions))]
    compile_error!("there is no production system that efficiently checks the source url yet.");
    gloo_net::http::Request::post(&*(SOURCE_URL.to_string() + path))
        .json(data)?
        .send().await?
        .json().await
}
pub async fn get_all_stati() -> Result<Vec<api_types::DataminerStatus>, gloo_net::Error> {
    post_request("/api/all_statuses", &()).await
}