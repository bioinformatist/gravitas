use gloo_net::http::Request;
use gravitas_core::types::OptionContract;

pub async fn fetch_options(api_url: &str, symbol: &str) -> Result<Vec<OptionContract>, String> {
    let url = format!("{api_url}/options/{symbol}");
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {e}"))?;

    if !resp.ok() {
        return Err(format!("API returned status {}", resp.status()));
    }

    let contracts: Vec<OptionContract> = resp
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {e}"))?;

    Ok(contracts)
}
