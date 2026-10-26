//! The PhotoCraft editor as a separate process, driven over its JSON control channel
//! (`photocraft/docs/control-protocol.md`): loopback TCP, one JSON line per request and
//! reply, and an `auth` frame with a token first.

use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use fude_core::control::Call;
use serde_json::{Value, json};

use crate::error::{AppError, AppResult};

/// How long to wait for a reply; PhotoCraft itself gives up after 60 s.
const REPLY_TIMEOUT: Duration = Duration::from_secs(70);
/// How long the editor may take to open its window and channel.
const START_TIMEOUT: Duration = Duration::from_secs(60);

pub struct Client {
    reader: BufReader<TcpStream>,
    writer: TcpStream,
    next: u64,
}

impl Client {
    /// Connects and authenticates.
    pub fn connect(addr: SocketAddr, token: &str) -> AppResult<Client> {
        let stream = TcpStream::connect_timeout(&addr, Duration::from_secs(2))?;
        stream.set_read_timeout(Some(REPLY_TIMEOUT))?;
        stream.set_nodelay(true)?;
        let writer = stream.try_clone()?;
        let mut client = Client { reader: BufReader::new(stream), writer, next: 1 };
        client.exchange(json!({ "id": "auth", "method": "auth", "params": { "token": token } }), &json!("auth"))?;
        Ok(client)
    }

    /// Sends one request and returns its `result`, or the editor's error.
    pub fn send(&mut self, method: &str, params: Value) -> AppResult<Value> {
        let id = self.next;
        self.next += 1;
        self.exchange(json!({ "id": id, "method": method, "params": params }), &json!(id))
    }

    pub fn call(&mut self, call: &Call) -> AppResult<Value> {
        self.send(&call.method, call.params.clone())
    }

    fn exchange(&mut self, request: Value, id: &Value) -> AppResult<Value> {
        let mut line = request.to_string();
        line.push('\n');
        self.writer.write_all(line.as_bytes())?;
        self.writer.flush()?;
        loop {
            let mut reply = String::new();
            if self.reader.read_line(&mut reply)? == 0 {
                return Err(AppError::msg("редактор закрыл соединение"));
            }
            let Ok(reply) = serde_json::from_str::<Value>(&reply) else { continue };
            if reply.get("id") != Some(id) {
                continue;
            }
            if reply.get("ok").and_then(Value::as_bool) == Some(true) {
                return Ok(reply.get("result").cloned().unwrap_or(Value::Null));
            }
            let error = reply.get("error").and_then(Value::as_str).unwrap_or("неизвестная ошибка");
            return Err(AppError::msg(format!("редактор: {error}")));
        }
    }
}

/// A fresh 256-bit token as 64 hex characters.
pub fn new_token() -> AppResult<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| AppError::msg(format!("нет случайных чисел: {e}")))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

pub fn free_port() -> AppResult<u16> {
    Ok(TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?.local_addr()?.port())
}

/// Where the PhotoCraft executable is: `FUDE_PHOTOCRAFT`, next to the app, or the
/// release build in the repository during development.
pub fn find_executable(beside: Option<&Path>) -> Option<PathBuf> {
    let name = if cfg!(windows) { "photocraft.exe" } else { "photocraft" };
    let mut candidates: Vec<PathBuf> = std::env::var_os("FUDE_PHOTOCRAFT").map(PathBuf::from).into_iter().collect();
    if let Some(dir) = beside {
        candidates.push(dir.join(name));
        candidates.push(dir.join("photocraft").join(name));
    }
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("photocraft").join("target");
    candidates.push(repo.join("release").join(name));
    candidates.push(repo.join("debug").join(name));
    candidates.into_iter().find(|p| p.is_file())
}

/// A running editor.
pub struct Editor {
    child: Child,
    client: Client,
    pub read_root: PathBuf,
    pub write_root: PathBuf,
}

impl Editor {
    /// Starts PhotoCraft with its channel on a free port and the two file roots, and waits
    /// until it answers.
    pub fn launch(exe: &Path, token_file: &Path, read_root: &Path, write_root: &Path) -> AppResult<Editor> {
        std::fs::create_dir_all(write_root)?;
        if let Some(parent) = token_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let token = new_token()?;
        std::fs::write(token_file, &token)?;
        let port = free_port()?;
        let mut child = Command::new(exe)
            .arg("--control")
            .arg(port.to_string())
            .arg("--control-token-file")
            .arg(token_file)
            .arg("--automation-read-root")
            .arg(read_root)
            .arg("--automation-write-root")
            .arg(write_root)
            .spawn()
            .map_err(|e| AppError::msg(format!("не удалось запустить редактор {}: {e}", exe.display())))?;
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        let started = Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                return Err(AppError::msg(format!("редактор закрылся при запуске ({status})")));
            }
            match Client::connect(addr, &token) {
                Ok(client) => return Ok(Editor { child, client, read_root: read_root.to_path_buf(), write_root: write_root.to_path_buf() }),
                Err(_) if started.elapsed() < START_TIMEOUT => std::thread::sleep(Duration::from_millis(300)),
                Err(e) => {
                    let _ = child.kill();
                    return Err(AppError::msg(format!("редактор не ответил за {} с: {e}", START_TIMEOUT.as_secs())));
                }
            }
        }
    }

    pub fn alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// The editor's process id, to find its window.
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn call(&mut self, call: &Call) -> AppResult<Value> {
        self.client.call(call)
    }

    pub fn calls(&mut self, calls: &[Call]) -> AppResult<()> {
        for call in calls {
            self.client.call(call)?;
        }
        Ok(())
    }

    pub fn inspect(&mut self) -> AppResult<Value> {
        self.client.call(&fude_core::control::inspect())
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::thread::JoinHandle;

    pub type Requests = Arc<Mutex<Vec<(String, Value)>>>;

    /// A stand-in for PhotoCraft's control server: checks the token, records requests and
    /// answers through `reply`.
    pub fn fake_editor(token: &'static str, reply: impl Fn(&str, &Value) -> Result<Value, String> + Send + 'static) -> (SocketAddr, Requests, JoinHandle<()>) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let addr = listener.local_addr().unwrap();
        let log = Arc::new(Mutex::new(Vec::new()));
        let seen = log.clone();
        let handle = std::thread::spawn(move || {
            let Ok((stream, _)) = listener.accept() else { return };
            let mut writer = stream.try_clone().unwrap();
            let reader = BufReader::new(stream);
            let mut authed = false;
            for line in reader.lines() {
                let Ok(line) = line else { return };
                let req: Value = serde_json::from_str(&line).unwrap();
                let id = req["id"].clone();
                let method = req["method"].as_str().unwrap_or_default().to_string();
                let answer = if method == "auth" {
                    authed = req["params"]["token"] == token;
                    if authed { json!({"id": id, "ok": true, "result": {}}) } else { json!({"id": id, "ok": false, "error": "bad token"}) }
                } else if !authed {
                    json!({"id": id, "ok": false, "error": "not authenticated"})
                } else {
                    seen.lock().unwrap().push((method.clone(), req["params"].clone()));
                    // Noise the client must skip: a reply for another id.
                    writeln!(writer, "{}", json!({"id": "other", "ok": true})).unwrap();
                    match reply(&method, &req["params"]) {
                        Ok(result) => json!({"id": id, "ok": true, "result": result}),
                        Err(error) => json!({"id": id, "ok": false, "error": error}),
                    }
                };
                writeln!(writer, "{answer}").unwrap();
            }
        });
        (addr, log, handle)
    }

    #[test]
    fn the_client_authenticates_and_matches_replies_by_id() {
        let (addr, log, _h) = fake_editor("secret", |method, params| match method {
            "ui.inspect" => Ok(json!({"tool": "Brush"})),
            "engine.execute" if params["command"] == "file.save" => Err("refused".into()),
            _ => Ok(json!(null)),
        });
        let mut client = Client::connect(addr, "secret").unwrap();
        assert_eq!(client.send("ui.inspect", json!({})).unwrap()["tool"], "Brush");
        let err = client.call(&Call::engine("file.save", json!({}))).unwrap_err().to_string();
        assert_eq!(err, "редактор: refused");
        assert_eq!(log.lock().unwrap().len(), 2);
    }

    #[test]
    fn a_wrong_token_is_refused() {
        let (addr, _log, _h) = fake_editor("secret", |_, _| Ok(json!(null)));
        let err = Client::connect(addr, "guess").err().unwrap().to_string();
        assert!(err.contains("bad token"), "{err}");
    }

    #[test]
    fn tokens_are_fresh_hex() {
        let (a, b) = (new_token().unwrap(), new_token().unwrap());
        assert_eq!(a.len(), 64);
        assert!(a.bytes().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
        assert!(free_port().unwrap() > 0);
    }

    #[test]
    fn a_missing_executable_fails_with_a_message() {
        let dir = tempfile::tempdir().unwrap();
        let err = Editor::launch(&dir.path().join("nope.exe"), &dir.path().join("t"), dir.path(), &dir.path().join("w")).err().unwrap().to_string();
        assert!(err.contains("не удалось запустить редактор"), "{err}");
    }
}
