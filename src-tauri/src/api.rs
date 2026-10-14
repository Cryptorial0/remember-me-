//! Local-only HTTP API (127.0.0.1) used by the browser extension.
use crate::api_core::route;
use crate::db::Db;
use std::io::Read;
use tauri::{AppHandle, Emitter, Manager};
use tiny_http::{Header, Method, Request, Response, Server};

const PORTS: std::ops::RangeInclusive<u16> = 47821..=47830;

fn h(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap()
}

fn handle(app: &AppHandle, mut req: Request, port: u16, token: &str) {
    let method = match req.method() {
        Method::Get => "GET",
        Method::Post => "POST",
        Method::Put => "PUT",
        Method::Delete => "DELETE",
        Method::Options => "OPTIONS",
        _ => "OTHER",
    };
    let path = req.url().split('?').next().unwrap_or("").to_string();

    let mut host_ok = false;
    let mut given = String::new();
    for hd in req.headers() {
        let name = hd.field.as_str().as_str().to_ascii_lowercase();
        let value = hd.value.as_str();
        if name == "host" {
            host_ok = value == format!("127.0.0.1:{port}") || value == format!("localhost:{port}");
        } else if name == "x-rm-token" {
            given = value.to_string();
        }
    }

    let mut body = String::new();
    if method == "POST" || method == "PUT" {
        let _ = req.as_reader().take(65_536).read_to_string(&mut body);
    }

    let db = app.state::<Db>();
    let r = route(&db, method, &path, host_ok, token, &given, &body);
    if r.changed {
        let _ = app.emit("changed", ());
    }
    let resp = Response::from_string(r.body)
        .with_status_code(r.status)
        .with_header(h("Content-Type", "application/json"))
        .with_header(h("Access-Control-Allow-Origin", "*"))
        .with_header(h("Access-Control-Allow-Headers", "Content-Type, X-RM-Token"))
        .with_header(h("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS"))
        .with_header(h("Access-Control-Allow-Private-Network", "true"));
    let _ = req.respond(resp);
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let db = app.state::<Db>();
        let token = match db.get_setting("api_token") {
            Ok(Some(t)) if !t.is_empty() => t,
            _ => match db.random_token() {
                Ok(t) => {
                    let _ = db.set_setting("api_token", &t);
                    t
                }
                Err(_) => return, // no safe token, no API
            },
        };
        let mut bound = None;
        for p in PORTS {
            if let Ok(s) = Server::http(("127.0.0.1", p)) {
                bound = Some((s, p));
                break;
            }
        }
        let Some((server, port)) = bound else { return };
        let _ = db.set_setting("api_port", &port.to_string());
        for req in server.incoming_requests() {
            handle(&app, req, port, &token);
        }
    });
}
