/// Minimal synchronous XRPC client backed by ureq.
///
/// Only implements the three calls needed for atproto publishing:
/// - `com.atproto.server.createSession`
/// - `com.atproto.repo.createRecord`
/// - `com.atproto.repo.putRecord`
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub access_jwt: String,
    pub did: String,
    pub handle: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordRef {
    pub uri: String,
}

/// Authenticate with a PDS using an app-password.
/// Calls `com.atproto.server.createSession`.
pub fn create_session(
    pds_url: &str,
    identifier: &str,
    password: &str,
) -> Result<Session, Box<dyn std::error::Error>> {
    #[derive(Serialize)]
    struct Body<'a> {
        identifier: &'a str,
        password: &'a str,
    }

    let url = format!("{pds_url}/xrpc/com.atproto.server.createSession");
    let body = Body {
        identifier,
        password,
    };
    let mut response = ureq::post(&url)
        .send_json(&body)
        .map_err(|e| format!("Authentication request failed: {e}"))?;

    let session: Session = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("Failed to parse session response: {e}"))?;

    Ok(session)
}

/// Create a new repository record.
/// Calls `com.atproto.repo.createRecord`.
pub fn create_record(
    pds_url: &str,
    access_jwt: &str,
    repo: &str,
    collection: &str,
    record: &serde_json::Value,
) -> Result<RecordRef, Box<dyn std::error::Error>> {
    #[derive(Serialize)]
    struct Body<'a> {
        repo: &'a str,
        collection: &'a str,
        record: &'a serde_json::Value,
        validate: bool,
    }

    let url = format!("{pds_url}/xrpc/com.atproto.repo.createRecord");
    let body = Body {
        repo,
        collection,
        record,
        validate: false,
    };
    let mut response = ureq::post(&url)
        .header("Authorization", &format!("Bearer {access_jwt}"))
        .send_json(&body)
        .map_err(|e| format!("createRecord request failed: {e}"))?;

    let result: RecordRef = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("Failed to parse createRecord response: {e}"))?;

    Ok(result)
}

/// Update an existing repository record.
/// Calls `com.atproto.repo.putRecord`.
pub fn put_record(
    pds_url: &str,
    access_jwt: &str,
    repo: &str,
    collection: &str,
    rkey: &str,
    record: &serde_json::Value,
) -> Result<RecordRef, Box<dyn std::error::Error>> {
    #[derive(Serialize)]
    struct Body<'a> {
        repo: &'a str,
        collection: &'a str,
        rkey: &'a str,
        record: &'a serde_json::Value,
        validate: bool,
    }

    let url = format!("{pds_url}/xrpc/com.atproto.repo.putRecord");
    let body = Body {
        repo,
        collection,
        rkey,
        record,
        validate: false,
    };
    let mut response = ureq::post(&url)
        .header("Authorization", &format!("Bearer {access_jwt}"))
        .send_json(&body)
        .map_err(|e| format!("putRecord request failed: {e}"))?;

    let result: RecordRef = response
        .body_mut()
        .read_json()
        .map_err(|e| format!("Failed to parse putRecord response: {e}"))?;

    Ok(result)
}

#[derive(Debug, Deserialize)]
struct DidDocumentService {
    id: String,
    #[serde(rename = "type")]
    service_type: String,
    #[serde(rename = "serviceEndpoint")]
    service_endpoint: String,
}

#[derive(Debug, Deserialize)]
struct DidDocument {
    service: Option<Vec<DidDocumentService>>,
}

/// Resolves an atproto handle to its PDS endpoint
pub fn resolve_pds_endpoint(handle: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Resolve handle to DID (via well-known or DNS)
    let did = resolve_handle_to_did(handle)?;

    // 2. Resolve DID to DID Document and find PDS
    let pds = resolve_did_to_pds(&did)?;
    Ok(pds)
}

fn resolve_handle_to_did(handle: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Try HTTPS .well-known lookup
    let well_known_url = format!("https://{handle}/.well-known/atproto-did");
    if let Ok(mut resp) = ureq::get(&well_known_url).call() {
        if resp.status().as_u16() == 200 {
            if let Ok(body) = resp.body_mut().read_to_string() {
                let did = body.trim().to_string();
                if did.starts_with("did:") {
                    return Ok(did);
                }
            }
        }
    }

    // Try Cloudflare DNS-over-HTTPS JSON query for TXT record fallback
    let dns_url = format!("https://cloudflare-dns.com/dns-query?name=_atproto.{handle}&type=TXT");
    if let Ok(mut resp) = ureq::get(&dns_url)
        .header("Accept", "application/dns-json")
        .call()
    {
        #[derive(Deserialize)]
        struct DnsAnswer {
            data: String,
        }
        #[derive(Deserialize)]
        struct DnsResponse {
            #[serde(rename = "Answer")]
            answer: Option<Vec<DnsAnswer>>,
        }

        if let Ok(dns_resp) = resp.body_mut().read_json::<DnsResponse>() {
            if let Some(answers) = dns_resp.answer {
                for ans in answers {
                    let txt = ans.data.trim_matches('"');
                    if let Some(stripped) = txt.strip_prefix("did=") {
                        return Ok(stripped.to_string());
                    }
                }
            }
        }
    }

    Err(format!("Could not resolve handle {handle} to a DID").into())
}

fn resolve_did_to_pds(did: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = if did.starts_with("did:plc:") {
        format!("https://plc.directory/{did}")
    } else if let Some(stripped) = did.strip_prefix("did:web:") {
        format!("https://{stripped}/.well-known/did.json")
    } else {
        return Err(format!("Unsupported DID method: {did}").into());
    };

    let mut resp = ureq::get(&url).call()?;
    let doc = resp.body_mut().read_json::<DidDocument>()?;
    if let Some(services) = doc.service {
        for service in services {
            if service.service_type == "AtprotoPersonalDataServer"
                || service.id.ends_with("#atproto_pds")
            {
                return Ok(service.service_endpoint);
            }
        }
    }
    Err("PDS endpoint not found in DID document".into())
}
