use std::io;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::header::{
    HeaderMap, HeaderName, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONNECTION,
};
use reqwest::Client;
use wireguard_keys::Privkey;

use crate::cli::Args;
use crate::registration::{CFResp, Registration, RegistrationResult};

mod cli;
mod registration;
mod wireguard_config;

const API_ENDPOINT: &str = "https://zero-trust-client.cloudflareclient.com/v0i2308311933/reg";
const INSTRUCTION_URL: &str = "https://github.com/poscat0x04/wgcf-teams/blob/master/guide.md";

#[tokio::main]
async fn main() -> Result<()> {
    let arg: Args = argh::from_env();

    let privkey = get_wg_privkey(arg.prompt)?;
    let token = get_jwt_token().await.context("Failed to get jwt token")?;

    let client = build_client()
        .await
        .context("Failed to build reqwest client")?;
    let reg = Registration::new(privkey, arg.device_name);
    let req = client
        .post(API_ENDPOINT)
        .json(&reg)
        .header("Cf-Access-Jwt-Assertion", token.trim())
        .build()
        .context("Failed to build request to cloudflare API")?;
    let raw_resp = client
        .execute(req)
        .await
        .context("Request to cloudflare API failed")?;
    let resp: CFResp<RegistrationResult> = raw_resp
        .json()
        .await
        .context("Failed to parse the result returned by cloudflare")?;
    let result = resp.get_result().context("Request failed")?;
    let wg_config = result.to_wg_config(privkey)?;

    println!("{wg_config}");
    Ok(())
}

pub async fn build_client() -> reqwest::Result<Client> {
    let mut hdr = HeaderMap::new();
    hdr.insert(
        ACCEPT_ENCODING,
        HeaderValue::from_str("gzip, deflate, br").unwrap(),
    );
    hdr.insert(
        ACCEPT_LANGUAGE,
        HeaderValue::from_str("en-US,en;q=0.9").unwrap(),
    );
    hdr.insert(ACCEPT, HeaderValue::from_str("*/*").unwrap());
    hdr.insert(CONNECTION, HeaderValue::from_str("keep-alive").unwrap());
    hdr.insert(
        HeaderName::from_bytes(b"CF-Client-Version").unwrap(),
        HeaderValue::from_str("i-6.23-2308311933.1").unwrap(),
    );
    Client::builder()
        .user_agent("1.1.1.1/6.23")
        .default_headers(hdr)
        .cookie_store(true)
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .timeout(Duration::from_secs(10))
        .build()
}

pub fn get_wg_privkey(prompt: bool) -> Result<Privkey> {
    if prompt {
        eprintln!("Please paste your wireguard private key and press enter:");
        let mut str = String::new();
        io::stdin()
            .read_line(&mut str)
            .context("Failed to read from stdin")?;
        Privkey::parse(str.trim_end()).context("Failed to parse wireguard private key")
    } else {
        Ok(Privkey::generate())
    }
}

pub async fn get_jwt_token() -> io::Result<String> {
    eprintln!("Please open https://<YOUR_ORGANIZATION>.cloudflareaccess.com/warp, log in to warp, paste the JWT token here and press enter.");
    eprintln!(
        "For a detailed instruction on where to find the JWT token after login, see {}.",
        INSTRUCTION_URL
    );
    let mut str = String::new();
    io::stdin().read_line(&mut str)?;
    Ok(str)
}
