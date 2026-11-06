//! The PhotoCraft editor as a separate process, driven over its JSON control channel
//! (`photocraft/docs/control-protocol.md`): loopback TCP, one JSON line per request and
//! reply, and an `auth` frame with a token first.

use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
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
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    fude_core::i18n::pick("редактор закрыл соединение", "the editor closed the connection"),
                )
                .into());
            }
            let Ok(reply) = serde_json::from_str::<Value>(&reply) else { continue };
            if reply.get("id") != Some(id) {
                continue;
            }
            if reply.get("ok").and_then(Value::as_bool) == Some(true) {
                return Ok(reply.get("result").cloned().unwrap_or(Value::Null));
            }
            let error = reply.get("error").and_then(Value::as_str).unwrap_or(fude_core::i18n::pick("неизвестная ошибка", "unknown error"));
            return Err(AppError::msg(fude_core::tr!("редактор: {error}", "editor: {error}")));
        }
    }
}

/// A channel that reconnects once when the editor has dropped the connection (PhotoCraft closes
/// connections that stay idle, e.g. between showing a hint and recording a guide).
pub struct Link {
    client: Client,
    addr: SocketAddr,
    token: String,
}

impl Link {
    pub fn connect(addr: SocketAddr, token: &str) -> AppResult<Link> {
        Ok(Link { client: Client::connect(addr, token)?, addr, token: token.to_string() })
    }

    pub fn call(&mut self, call: &Call) -> AppResult<Value> {
        match self.client.call(call) {
            Err(e) if dropped(&e) => {
                self.client = Client::connect(self.addr, &self.token)?;
                self.client.call(call)
            }
            other => other,
        }
    }
}

/// The connection is gone, so the request never ran and may be sent again.
fn dropped(e: &AppError) -> bool {
    use std::io::ErrorKind::{BrokenPipe, ConnectionAborted, ConnectionReset, NotConnected, UnexpectedEof};
    matches!(e, AppError::Io(io) if matches!(io.kind(), ConnectionAborted | ConnectionReset | BrokenPipe | NotConnected | UnexpectedEof))
}

/// What the editor said about its control channel on its standard error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlLine {
    Listening(SocketAddr),
    Failed(String),
}

/// Reads PhotoCraft's channel lines: `photocraft: control server listening on 127.0.0.1:51150`
/// or `photocraft: control server failed to bind 127.0.0.1:0: <reason>`.
pub fn parse_control_line(line: &str) -> Option<ControlLine> {
    let rest = line.trim().strip_prefix("photocraft: control server ")?;
    if let Some(addr) = rest.strip_prefix("listening on ") {
        return addr.trim().parse().ok().map(ControlLine::Listening);
    }
    rest.strip_prefix("failed to bind ").map(|reason| ControlLine::Failed(reason.trim().to_string()))
}

/// A fresh 256-bit token as 64 hex characters.
pub fn new_token() -> AppResult<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| AppError::msg(fude_core::tr!("нет случайных чисел: {e}", "no random numbers: {e}")))?;
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
    client: Link,
    pub read_root: PathBuf,
    pub write_root: PathBuf,
}

impl Editor {
    /// Starts PhotoCraft with its channel on a free port and the two file roots, and waits
    /// until it answers.
    pub fn launch(exe: &Path, token_file: &Path, read_root: &Path, write_root: &Path) -> AppResult<Editor> {
        // Port 0: PhotoCraft binds a free port itself and reports it. Choosing one here and
        // handing it over races with every other program that opens a port in between: when the
        // editor lost that race it ran on without its channel and FUDE gave up with a busy port.
        let mut command = Command::new(exe);
        command
            .arg("--control")
            .arg("0")
            .arg("--control-token-file")
            .arg(token_file)
            .arg("--automation-read-root")
            .arg(read_root)
            .arg("--automation-write-root")
            .arg(write_root);
        Editor::start(command, token_file, read_root, write_root, START_TIMEOUT)
    }

    /// Runs `command` (an editor told to write its channel line to standard error), writing a
    /// fresh token to `token_file` first, and connects once the editor reports its port.
    pub fn start(mut command: Command, token_file: &Path, read_root: &Path, write_root: &Path, timeout: Duration) -> AppResult<Editor> {
        std::fs::create_dir_all(write_root)?;
        if let Some(parent) = token_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let token = new_token()?;
        std::fs::write(token_file, &token)?;
        let program = command.get_program().to_string_lossy().into_owned();
        let mut child = command
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::msg(fude_core::tr!("не удалось запустить редактор {program}: {e}", "could not start the editor {program}: {e}")))?;
        let (tx, rx) = mpsc::channel();
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                // Pass the editor's log through, and report the first channel line.
                let mut tx = Some(tx);
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    eprintln!("{line}");
                    if let Some(found) = parse_control_line(&line)
                        && let Some(tx) = tx.take()
                    {
                        let _ = tx.send(found);
                    }
                }
            });
        }
        let started = Instant::now();
        let addr = loop {
            if let Some(status) = child.try_wait()? {
                return Err(AppError::msg(fude_core::tr!("редактор закрылся при запуске ({status})", "the editor closed while starting ({status})")));
            }
            match rx.recv_timeout(Duration::from_millis(200)) {
                Ok(ControlLine::Listening(addr)) => break addr,
                Ok(ControlLine::Failed(reason)) => {
                    let _ = child.kill();
                    return Err(AppError::msg(fude_core::tr!(
                        "редактор не смог открыть канал управления: {reason}",
                        "the editor could not open its control channel: {reason}"
                    )));
                }
                Err(_) if started.elapsed() < timeout => {}
                Err(_) => {
                    let _ = child.kill();
                    return Err(AppError::msg(fude_core::tr!(
                        "редактор не открыл канал управления за {} с",
                        "the editor did not open its control channel within {} s",
                        timeout.as_secs()
                    )));
                }
            }
        };
        // The channel is listening; the first connection can still race the editor's startup.
        let mut last = None;
        for _ in 0..25 {
            match Link::connect(addr, &token) {
                Ok(client) => return Ok(Editor { child, client, read_root: read_root.to_path_buf(), write_root: write_root.to_path_buf() }),
                Err(e) => {
                    last = Some(e);
                    std::thread::sleep(Duration::from_millis(200));
                }
            }
        }
        let _ = child.kill();
        Err(AppError::msg(fude_core::tr!(
            "редактор не ответил на {addr}: {}",
            "the editor did not answer at {addr}: {}",
            last.map_or_else(String::new, |e| e.to_string())
        )))
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
            self.call(call)?;
        }
        Ok(())
    }

    pub fn inspect(&mut self) -> AppResult<Value> {
        self.call(&fude_core::control::inspect())
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
    fn a_dropped_connection_is_reopened_once() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let addr = listener.local_addr().unwrap();
        let served = std::thread::spawn(move || {
            // First connection: authenticate, then hang up instead of answering, like an
            // editor that closed an idle channel. Second: answer normally.
            for round in 0..2 {
                let (stream, _) = listener.accept().unwrap();
                let mut writer = stream.try_clone().unwrap();
                let mut lines = BufReader::new(stream).lines();
                let auth: Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
                writeln!(writer, "{}", json!({"id": auth["id"], "ok": true, "result": {}})).unwrap();
                let req: Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
                if round == 1 {
                    writeln!(writer, "{}", json!({"id": req["id"], "ok": true, "result": {"round": round}})).unwrap();
                }
            }
        });
        let mut link = Link::connect(addr, "t").unwrap();
        let reply = link.call(&Call::new("ui.inspect", json!({}))).unwrap();
        assert_eq!(reply["round"], 1, "answered on the new connection");
        served.join().unwrap();
        assert!(dropped(&std::io::Error::from(std::io::ErrorKind::ConnectionAborted).into()));
        assert!(!dropped(&AppError::msg("редактор: нет такого слоя")), "the editor's own refusals are not retried");
    }

    #[test]
    fn a_wrong_token_is_refused() {
        let (addr, _log, _h) = fake_editor("secret", |_, _| Ok(json!(null)));
        let err = Client::connect(addr, "guess").err().unwrap().to_string();
        assert!(err.contains("bad token"), "{err}");
    }

    /// Plays PhotoCraft when this test binary is started by [`fake`]: reports its channel the way
    /// PhotoCraft does and answers requests. Run normally it does nothing.
    #[test]
    fn fake_editor_process() {
        let Ok(mode) = std::env::var("FUDE_FAKE_EDITOR") else { return };
        match mode.as_str() {
            "exit" => std::process::exit(3),
            "fail" => {
                eprintln!("photocraft: control server failed to bind 127.0.0.1:0: test refusal");
                std::thread::sleep(Duration::from_secs(30));
            }
            "silent" => std::thread::sleep(Duration::from_secs(30)),
            _ => {
                let token = std::fs::read_to_string(std::env::var("FUDE_FAKE_TOKEN").unwrap()).unwrap();
                let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
                eprintln!("photocraft: control token file: somewhere");
                eprintln!("photocraft: control server listening on {}", listener.local_addr().unwrap());
                let (stream, _) = listener.accept().unwrap();
                let mut writer = stream.try_clone().unwrap();
                for line in BufReader::new(stream).lines().map_while(Result::ok) {
                    let req: Value = serde_json::from_str(&line).unwrap();
                    let ok = req["method"] != "auth" || req["params"]["token"] == token.as_str();
                    let reply = json!({"id": req["id"], "ok": ok, "result": {"method": req["method"]}, "error": "bad token"});
                    writeln!(writer, "{reply}").unwrap();
                }
            }
        }
        std::process::exit(0);
    }

    /// This test binary, started as a fake editor in `mode`.
    fn fake(mode: &str, token: &Path) -> Command {
        let mut c = Command::new(std::env::current_exe().unwrap());
        c.args(["editor::tests::fake_editor_process", "--exact", "--nocapture", "--test-threads=1"])
            .env("FUDE_FAKE_EDITOR", mode)
            .env("FUDE_FAKE_TOKEN", token);
        c
    }

    #[test]
    fn an_editor_is_reached_on_the_port_it_reports() {
        let dir = tempfile::tempdir().unwrap();
        let token = dir.path().join("token");
        let mut editor = Editor::start(fake("ok", &token), &token, dir.path(), &dir.path().join("w"), Duration::from_secs(20)).unwrap();
        assert!(editor.alive());
        assert!(editor.pid() > 0);
        assert_eq!(editor.inspect().unwrap()["method"], "ui.inspect");
        assert_eq!(editor.call(&Call::engine("edit.undo", json!({}))).unwrap()["method"], "engine.execute");
        assert!(dir.path().join("w").is_dir(), "the write root is created");
        assert_eq!(std::fs::read_to_string(&token).unwrap().len(), 64, "a fresh token was written for the editor");
    }

    #[test]
    fn an_editor_that_cannot_open_its_channel_is_reported_at_once() {
        let dir = tempfile::tempdir().unwrap();
        let token = dir.path().join("token");
        let started = Instant::now();
        let err = Editor::start(fake("fail", &token), &token, dir.path(), dir.path(), Duration::from_secs(20)).err().unwrap().to_string();
        assert!(err.contains("не смог открыть канал управления") && err.contains("test refusal"), "{err}");
        assert!(started.elapsed() < Duration::from_secs(15), "no waiting for the timeout");
    }

    #[test]
    fn an_editor_that_exits_at_start_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let token = dir.path().join("token");
        let err = Editor::start(fake("exit", &token), &token, dir.path(), dir.path(), Duration::from_secs(20)).err().unwrap().to_string();
        assert!(err.contains("закрылся при запуске"), "{err}");
    }

    #[test]
    fn a_silent_editor_times_out() {
        let dir = tempfile::tempdir().unwrap();
        let token = dir.path().join("token");
        let err = Editor::start(fake("silent", &token), &token, dir.path(), dir.path(), Duration::from_secs(2)).err().unwrap().to_string();
        assert!(err.contains("не открыл канал управления за 2 с"), "{err}");
    }

    #[test]
    fn the_editor_is_found_beside_the_app() {
        let name = if cfg!(windows) { "photocraft.exe" } else { "photocraft" };
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("photocraft")).unwrap();
        std::fs::write(dir.path().join("photocraft").join(name), b"").unwrap();
        if std::env::var_os("FUDE_PHOTOCRAFT").is_none() {
            assert_eq!(find_executable(Some(dir.path())), Some(dir.path().join("photocraft").join(name)), "in a photocraft folder");
            std::fs::write(dir.path().join(name), b"").unwrap();
            assert_eq!(find_executable(Some(dir.path())), Some(dir.path().join(name)), "right beside fude.exe wins");
        }
    }

    #[test]
    fn the_editors_channel_line_gives_its_port() {
        assert_eq!(
            parse_control_line("photocraft: control server listening on 127.0.0.1:51150"),
            Some(ControlLine::Listening(SocketAddr::from((Ipv4Addr::LOCALHOST, 51150))))
        );
        assert_eq!(
            parse_control_line("  photocraft: control server listening on 127.0.0.1:7878\r"),
            Some(ControlLine::Listening(SocketAddr::from((Ipv4Addr::LOCALHOST, 7878))))
        );
        assert_eq!(
            parse_control_line("photocraft: control server failed to bind 127.0.0.1:0: access denied"),
            Some(ControlLine::Failed("127.0.0.1:0: access denied".into()))
        );
        for other in ["photocraft: control token file: C:\\x", "INFO [main] photocraft: GPU startup", "photocraft: control server listening on nowhere", ""] {
            assert_eq!(parse_control_line(other), None, "{other}");
        }
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
