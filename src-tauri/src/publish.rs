// Publishing helpers: a CORS-free HTTP command (used for all destination APIs and
// OAuth token exchange) and a tiny localhost loopback to capture OAuth redirects.
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct HttpReq {
    method: String,
    url: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
}

#[derive(Serialize)]
pub struct HttpResp {
    status: u16,
    body: String,
}

fn http_blocking(req: HttpReq) -> Result<HttpResp, String> {
    let mut r = ureq::request(&req.method.to_uppercase(), &req.url);
    if let Some(h) = &req.headers {
        for (k, v) in h {
            r = r.set(k, v);
        }
    }
    let result = match req.body {
        Some(b) => r.send_string(&b),
        None => r.call(),
    };
    match result {
        Ok(res) => {
            let status = res.status();
            let body = res.into_string().unwrap_or_default();
            Ok(HttpResp { status, body })
        }
        // Non-2xx still returns a response — surface its status + body to the frontend.
        Err(ureq::Error::Status(code, res)) => {
            let body = res.into_string().unwrap_or_default();
            Ok(HttpResp { status: code, body })
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn http_request(req: HttpReq) -> Result<HttpResp, String> {
    tauri::async_runtime::spawn_blocking(move || http_blocking(req))
        .await
        .map_err(|e| e.to_string())?
}

// ---- OAuth localhost loopback (captures ?code= from the provider redirect) ----
static OAUTH_CODE: Mutex<Option<String>> = Mutex::new(None);

#[tauri::command]
pub fn oauth_listen(port: u16) -> Result<(), String> {
    *OAUTH_CODE.lock().unwrap() = None;
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|e| format!("Could not bind localhost:{} — {}", port, e))?;
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]);
            let code = req
                .lines()
                .next()
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|path| path.split('?').nth(1))
                .and_then(|q| q.split('&').find(|kv| kv.starts_with("code=")))
                .map(|kv| kv["code=".len()..].to_string());
            if let Some(c) = code {
                *OAUTH_CODE.lock().unwrap() = Some(c);
            }
            let html = "<html><body style=\"font-family:sans-serif;text-align:center;padding:64px;background:#EAE6D9;color:#15110A\"><h2>ContentOS — connected.</h2><p>You can close this tab and return to the app.</p></body></html>";
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                html.len(),
                html
            );
            let _ = stream.write_all(resp.as_bytes());
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn oauth_await() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(|| {
        for _ in 0..600 {
            if let Some(c) = OAUTH_CODE.lock().unwrap().clone() {
                return Ok(c);
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        Err("Timed out waiting for authorization (2 min).".to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}
