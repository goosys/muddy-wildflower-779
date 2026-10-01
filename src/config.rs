use axum::http::Uri;
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SiteUrls {
    url: String,
    workers_dev_url: String,
}

pub(crate) static PUBLIC_ACCESS: LazyLock<AccessConfig> = LazyLock::new(|| {
    let site: SiteUrls = serde_json::from_str(include_str!("../frontend/src/content/site.json"))
        .expect("site configuration must be valid JSON");
    AccessConfig::new(&site.url, &site.workers_dev_url)
        .expect("site configuration must contain valid public URLs")
});

/// Public MCP endpoints and the trusted Workers preview namespace.
#[derive(Debug, Clone)]
pub struct AccessConfig {
    pub(crate) public_hosts: [String; 2],
    preview_suffix: String,
    max_preview_prefix_length: usize,
}

impl AccessConfig {
    /// Derive MCP access rules from the configured public URLs.
    ///
    /// # Errors
    /// Returns an error for non-HTTPS origins or an invalid Workers hostname.
    pub fn new(public_url: &str, workers_dev_url: &str) -> Result<Self, &'static str> {
        let public_host = https_host(public_url)?;
        let workers_host = https_host(workers_dev_url)?;
        let namespace = workers_host
            .strip_suffix(".workers.dev")
            .ok_or("workersDevUrl must use <worker>.<account>.workers.dev")?;
        let (worker, account) = namespace
            .split_once('.')
            .ok_or("workersDevUrl must include a Worker and account")?;
        if !valid_dns_label(worker) || !valid_dns_label(account) {
            return Err("workersDevUrl must include exactly one Worker and account label");
        }
        let max_preview_prefix_length = 63_usize.saturating_sub(worker.len() + 1);
        let preview_suffix = format!("-{workers_host}");
        Ok(Self {
            public_hosts: [public_host, workers_host],
            preview_suffix,
            max_preview_prefix_length,
        })
    }

    pub(crate) fn preview_host(&self, authority: &str) -> Option<String> {
        let authority = authority.to_ascii_lowercase();
        let host = authority.strip_suffix(":443").unwrap_or(&authority);
        let prefix = host.strip_suffix(&self.preview_suffix)?;
        if prefix.len() > self.max_preview_prefix_length || !valid_dns_label(prefix) {
            return None;
        }
        Some(host.to_owned())
    }
}

fn https_host(value: &str) -> Result<String, &'static str> {
    let uri: Uri = value
        .parse()
        .map_err(|_| "public URL must be a valid URI")?;
    if uri.scheme_str() != Some("https")
        || uri.path() != "/"
        || uri.query().is_some()
        || value.contains('#')
    {
        return Err("public URL must be an HTTPS origin without a path, query, or fragment");
    }
    let authority = uri
        .authority()
        .ok_or("public URL must include a hostname")?;
    let port_suffix = authority.as_str().strip_prefix(authority.host());
    if authority.as_str().contains('@') || !matches!(port_suffix, Some("" | ":443")) {
        return Err("public URL must omit credentials and use HTTPS port 443");
    }
    let host = authority.host().to_ascii_lowercase();
    if !host.contains('.')
        || host.len() > 253
        || !host.split('.').all(valid_dns_label)
        || host.parse::<std::net::IpAddr>().is_ok()
    {
        return Err("public URL must use a valid DNS hostname");
    }
    Ok(host)
}

fn valid_dns_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}
