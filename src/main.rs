//! restloop: a minimal agent loop. One model, one shell tool.
//!
//! The CLI: the help text, the settings of the environment, the session and the
//! prompts. The loop itself is the library's.

use restloop::Config;
use serde_json::{Value, json};
use std::env;
use std::io::BufRead;
use std::time::{Duration, Instant};

/// The steps a task may take before the loop stops.
const MAX_STEPS: u64 = 100;
/// The seconds a command may run.
const DEFAULT_TIMEOUT: u64 = 120;

fn main() {
    let start = Instant::now();
    let task = env::args().skip(1).collect::<Vec<_>>().join(" ");
    if task == "-h" || task == "--help" {
        println!(
            "usage: restloop [task]   (no task: read tasks from stdin)\n\nenv: RESTLOOP_MODEL (required), RESTLOOP_API_KEY, RESTLOOP_BASE_URL (default https://api.openai.com/v1),\n     RESTLOOP_API_KEY_FILE (file holding the key, read for every request),\n     RESTLOOP_WIRE_API=responses (use /responses instead of /chat/completions),\n     RESTLOOP_REASONING_EFFORT (e.g. low, medium, high),\n     RESTLOOP_TIMEOUT (seconds per command, default 120), RESTLOOP_MAX_STEPS (default 100),\n     RESTLOOP_LOG_USAGE=1 (token usage of each model reply on stderr),\n     RESTLOOP_DEADLINE (seconds for the whole task; the model is told the time used),\n     RESTLOOP_LAST_MESSAGE (file that receives the final reply),\n     RESTLOOP_SESSION (file to continue the conversation from; saved after every step)"
        );
        return;
    }
    let Some(cfg) = config(|k| env::var(k).ok(), start) else {
        eprintln!("restloop: set RESTLOOP_MODEL (see --help)");
        std::process::exit(2);
    };
    // The commands of the run must not outlive its deadline.
    let limit = cfg
        .deadline
        .and_then(|d| cfg.start.checked_add(Duration::from_secs(d)));
    restloop::shell::install(limit);
    let mut messages = match conversation(&cfg) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("restloop: {e}");
            std::process::exit(2);
        }
    };
    let resumed = messages.len() > 1;
    let ask = |text: &str, messages: &mut Vec<Value>| {
        // A resumed conversation keeps its system prompt, so the provider's cache still
        // matches; this run's time budget comes with the new message instead.
        let text = match cfg.deadline {
            Some(d) if resumed => format!(
                "{text}\n\n[You have {d}s in total for this request; the time used in tool results counts from now.]"
            ),
            _ => text.to_string(),
        };
        messages.push(json!({"role": "user", "content": text}));
        let run = restloop::run(&cfg, messages, &mut |cmd, secs| {
            restloop::shell::shell(cmd, secs)
        });
        // Nothing a run started may keep running after the run is over.
        restloop::shell::stop_all();
        if let Err(e) = run {
            eprintln!("restloop: {e}");
            return false;
        }
        true
    };
    if !task.is_empty() {
        let ok = ask(&task, &mut messages);
        std::process::exit(if ok { 0 } else { 1 });
    }
    let stdin = std::io::stdin();
    loop {
        eprint!("> ");
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        if !line.trim().is_empty() {
            ask(line.trim(), &mut messages);
        }
    }
}

/// The settings the environment holds: None when RESTLOOP_MODEL is missing.
/// The getter lives in the key closure, so it has to live as long as the settings
/// and to be safe to share between the threads of a run.
fn config(
    get: impl Fn(&str) -> Option<String> + Send + Sync + 'static,
    start: Instant,
) -> Option<Config> {
    let model = get("RESTLOOP_MODEL")?;
    let base_url = get("RESTLOOP_BASE_URL").unwrap_or_else(|| "https://api.openai.com/v1".into());
    let responses = get("RESTLOOP_WIRE_API").is_some_and(|v| v == "responses");
    let reasoning_effort = get("RESTLOOP_REASONING_EFFORT").filter(|v| !v.is_empty());
    let max_steps = env_num(&get, "RESTLOOP_MAX_STEPS", MAX_STEPS);
    let timeout = env_num(&get, "RESTLOOP_TIMEOUT", DEFAULT_TIMEOUT);
    let deadline = get("RESTLOOP_DEADLINE").and_then(|v| v.parse().ok());
    let log_usage = get("RESTLOOP_LOG_USAGE").is_some_and(|v| v == "1");
    let last_message = get("RESTLOOP_LAST_MESSAGE");
    let session = get("RESTLOOP_SESSION").filter(|p| !p.is_empty());
    Some(Config {
        model,
        base_url,
        // The key is read for every request, so it stays a closure of its own: it
        // holds the getter and nothing else.
        key: Box::new(move || key(&get)),
        responses,
        reasoning_effort,
        max_steps,
        timeout,
        deadline,
        start,
        log_usage,
        last_message,
        session,
    })
}

/// The messages of a saved session, if the text holds one.
fn parse_session(text: &str) -> Option<Vec<Value>> {
    let doc: Value = serde_json::from_str(text).unwrap_or_default();
    match doc["messages"].as_array() {
        Some(m) if doc["version"] == 1 && !m.is_empty() => Some(m.clone()),
        _ => None,
    }
}

/// The conversation saved in RESTLOOP_SESSION if that file exists, else a new one.
/// A file that holds no session is an error the CLI reports.
fn conversation(cfg: &Config) -> Result<Vec<Value>, String> {
    let path = cfg.session.clone().unwrap_or_default();
    if path.is_empty() || !std::path::Path::new(&path).exists() {
        return Ok(vec![
            json!({"role": "system", "content": restloop::system_prompt(cfg, shell_name(), env::consts::OS, &cwd())}),
        ]);
    }
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    match parse_session(&text) {
        Some(m) => Ok(m),
        None => Err(format!("{path} is not a restloop session")),
    }
}

/// The name of the shell the commands run in, for the system prompt.
fn shell_name() -> &'static str {
    match restloop::shell::shell_path() {
        Some(_) if cfg!(windows) => "bash (Git Bash)",
        Some(_) => "sh",
        None => "PowerShell",
    }
}

/// The working directory the commands run in, for the system prompt.
fn cwd() -> String {
    env::current_dir().unwrap_or_default().display().to_string()
}

/// The key of a request: the file if there is one, else the variable.
fn key(get: impl Fn(&str) -> Option<String>) -> Result<String, String> {
    match get("RESTLOOP_API_KEY_FILE") {
        Some(p) if !p.is_empty() => {
            std::fs::read_to_string(&p).map_err(|e| format!("cannot read {p}: {e}"))
        }
        _ => Ok(get("RESTLOOP_API_KEY").unwrap_or_default()),
    }
}

/// A number from the environment, or the default when unset or invalid.
fn env_num(get: &impl Fn(&str) -> Option<String>, name: &str, default: u64) -> u64 {
    get(name).and_then(|v| v.parse().ok()).unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::{Mutex, MutexGuard};

    /// The lock the tests that read the environment hold: the environment belongs
    /// to the whole process, so only one of them may change it at a time.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// The lock of the environment, held for as long as it lives.
    fn env_lock() -> MutexGuard<'static, ()> {
        // A test that failed must not lock the others out: the lock is taken anyway.
        ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The path a directory of a test has in the base dir: the name of the test,
    /// the pid of the process and the number of the attempt. Another run may have
    /// used the path already, so the guard has to be careful with it.
    fn dir_path(base: &std::path::Path, name: &str, n: u64) -> std::path::PathBuf {
        base.join(format!("restloop-{name}-{}-{n}", std::process::id()))
    }

    /// A directory of a test in the base dir, made for the test alone: the path
    /// is one that nobody else has, so the test never takes a directory of
    /// another run.
    fn make_temp_dir(base: &std::path::Path, name: &str) -> std::path::PathBuf {
        for n in 0..1000 {
            let path = dir_path(base, name, n);
            // The directory is made in one step: create_dir fails when the path
            // is taken, so the directory of the test is never the one of another
            // run, and nothing is taken away.
            match std::fs::create_dir(&path) {
                Ok(()) => return path,
                // The path is taken: another name is asked for.
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("cannot make the directory of the test at {path:?}: {e}"),
            }
        }
        panic!("cannot make a directory of the test in {base:?}");
    }

    /// A file of a test, removed when the test ends: an assertion that fails must
    /// leave nothing behind. The file is made in a directory of the test alone
    /// and with create_new, so the guard writes a file of its own and never takes
    /// one that the test did not write.
    struct TempFile {
        // The directory of the test lives as long as the file: it is the guard
        // that takes the directory away when the file is gone.
        _dir: TempDir,
        path: std::path::PathBuf,
    }

    impl TempFile {
        /// A file of the test in a directory of its own, named after the test,
        /// holding the text the test needs.
        fn new(name: &str, text: &str) -> Self {
            Self::in_dir(&env::temp_dir(), name, text)
        }

        /// A file of the test in a directory of its own in the base dir, named
        /// after the test, holding the text the test needs.
        fn in_dir(base: &std::path::Path, name: &str, text: &str) -> Self {
            let dir = TempDir::in_dir(base, name);
            let path = dir.0.join(name);
            // create_new fails when the file is there: the guard never writes
            // over a file of anyone else.
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap();
            file.write_all(text.as_bytes()).unwrap();
            Self { _dir: dir, path }
        }

        /// The path of the file of the test.
        fn path(&self) -> &std::path::Path {
            &self.path
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            // The file is the one the guard made: it goes when the test ends.
            let _ = std::fs::remove_file(&self.path);
        }
    }

    /// A directory of a test, removed when the test ends: an assertion that fails
    /// must leave nothing behind. The directory is made for the test alone, so
    /// the guard takes the directory it made, not one of anyone else. A path that
    /// is not there is of a directory like this one: the guard takes the
    /// directory it made, not a file of someone else.
    struct TempDir(std::path::PathBuf);

    impl TempDir {
        /// A directory of the test in the temp dir, named after the test.
        fn new(name: &str) -> Self {
            Self::in_dir(&env::temp_dir(), name)
        }

        /// A directory of the test in the base dir, named after the test.
        fn in_dir(base: &std::path::Path, name: &str) -> Self {
            Self(make_temp_dir(base, name))
        }

        /// A path in the directory that holds no file: a test that needs a path
        /// that is not there takes one from here.
        fn missing(&self, name: &str) -> std::path::PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            // The directory is the one the guard made: it goes when the test ends.
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// A getter that answers from a list of settings.
    fn get(settings: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + Send + Sync + 'static {
        let settings: Vec<(String, String)> = settings
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| {
            settings
                .iter()
                .find(|(n, _)| n == k)
                .map(|(_, v)| v.clone())
        }
    }

    #[test]
    fn no_model_means_no_settings() {
        assert!(config(get(&[]), Instant::now()).is_none());
    }

    #[test]
    fn the_settings_have_the_defaults_the_cli_uses() {
        let start = Instant::now();
        let cfg = config(get(&[("RESTLOOP_MODEL", "m")]), start).unwrap();
        assert_eq!(cfg.model, "m");
        assert_eq!(cfg.base_url, "https://api.openai.com/v1");
        assert_eq!(cfg.max_steps, MAX_STEPS);
        assert_eq!(cfg.timeout, DEFAULT_TIMEOUT);
        assert_eq!(cfg.deadline, None);
        assert_eq!(cfg.reasoning_effort, None);
        assert!(!cfg.responses);
        assert!(!cfg.log_usage);
        assert_eq!(cfg.last_message, None);
        assert_eq!(cfg.session, None);
        assert_eq!(cfg.start, start);
    }

    #[test]
    fn every_setting_is_read_from_the_environment() {
        let cfg = config(
            get(&[
                ("RESTLOOP_MODEL", "m"),
                ("RESTLOOP_BASE_URL", "http://127.0.0.1:1/v1"),
                ("RESTLOOP_WIRE_API", "responses"),
                ("RESTLOOP_REASONING_EFFORT", "low"),
                ("RESTLOOP_MAX_STEPS", "5"),
                ("RESTLOOP_TIMEOUT", "7"),
                ("RESTLOOP_DEADLINE", "60"),
                ("RESTLOOP_LOG_USAGE", "1"),
                ("RESTLOOP_LAST_MESSAGE", "last.txt"),
                ("RESTLOOP_SESSION", "session.json"),
            ]),
            Instant::now(),
        )
        .unwrap();
        assert_eq!(cfg.base_url, "http://127.0.0.1:1/v1");
        assert!(cfg.responses);
        assert_eq!(cfg.reasoning_effort.as_deref(), Some("low"));
        assert_eq!(cfg.max_steps, 5);
        assert_eq!(cfg.timeout, 7);
        assert_eq!(cfg.deadline, Some(60));
        assert!(cfg.log_usage);
        assert_eq!(cfg.last_message.as_deref(), Some("last.txt"));
        assert_eq!(cfg.session.as_deref(), Some("session.json"));
    }

    #[test]
    fn an_empty_reasoning_effort_is_no_effort() {
        let cfg = config(
            get(&[("RESTLOOP_MODEL", "m"), ("RESTLOOP_REASONING_EFFORT", "")]),
            Instant::now(),
        )
        .unwrap();
        assert_eq!(cfg.reasoning_effort, None);
    }

    #[test]
    fn a_number_that_is_not_one_is_the_default() {
        let cfg = config(
            get(&[
                ("RESTLOOP_MODEL", "m"),
                ("RESTLOOP_MAX_STEPS", "many"),
                ("RESTLOOP_TIMEOUT", ""),
                ("RESTLOOP_DEADLINE", "soon"),
            ]),
            Instant::now(),
        )
        .unwrap();
        assert_eq!(cfg.max_steps, MAX_STEPS);
        assert_eq!(cfg.timeout, DEFAULT_TIMEOUT);
        assert_eq!(cfg.deadline, None);
    }

    #[test]
    fn a_saved_session_is_read_when_it_holds_one() {
        let messages = vec![json!({"role": "user", "content": "hi"})];
        let text = json!({"version": 1, "messages": messages}).to_string();
        assert_eq!(parse_session(&text), Some(messages));
    }

    /// The key of a request: the file if there is one, else the variable.
    #[test]
    fn the_key_comes_from_the_file_when_there_is_one() {
        // The directory of the test comes from the environment: another test must
        // not change it while this one runs, so the shared env lock is held for
        // the whole body.
        let _env = env_lock();
        let temp = TempFile::new("key-file", "from the file");
        let file = temp.path().to_string_lossy().to_string();
        let key = key(get(&[
            ("RESTLOOP_API_KEY_FILE", file.as_str()),
            ("RESTLOOP_API_KEY", "from the variable"),
        ]))
        .unwrap();
        assert_eq!(key, "from the file");
    }

    /// A key file that is not named is no file: the variable answers.
    #[test]
    fn an_empty_key_file_setting_falls_back_to_the_variable() {
        let key = key(get(&[
            ("RESTLOOP_API_KEY_FILE", ""),
            ("RESTLOOP_API_KEY", "from the variable"),
        ]))
        .unwrap();
        assert_eq!(key, "from the variable");
    }

    /// Without a key anywhere the key is empty.
    #[test]
    fn without_a_key_the_key_is_empty() {
        assert_eq!(key(get(&[])).unwrap(), "");
    }

    /// A key file that cannot be read is an error, not an empty key.
    #[test]
    fn a_key_file_that_cannot_be_read_is_an_error() {
        // The directory of the test comes from the environment: another test must
        // not change it while this one runs, so the shared env lock is held for
        // the whole body.
        let _env = env_lock();
        // The file is not there: the path is of the directory of the test, so the
        // guard of the test cannot take a file that is not its own.
        let dir = TempDir::new("no-key");
        let file = dir.missing("key-file").to_string_lossy().to_string();
        let e = key(get(&[("RESTLOOP_API_KEY_FILE", file.as_str())])).unwrap_err();
        assert!(e.contains("cannot read"), "{e}");
        assert!(e.contains(&file), "{e}");
    }

    /// A session file of another version is no session: the CLI says so and stops.
    #[test]
    fn a_session_file_that_is_not_a_session_is_an_error() {
        // The directory of the test comes from the environment: another test must
        // not change it while this one runs, so the shared env lock is held for
        // the whole body.
        let _env = env_lock();
        let temp = TempFile::new(
            "bad-session",
            &json!({"version": 2, "messages": [{"role": "user", "content": "hi"}]}).to_string(),
        );
        let file = temp.path().to_string_lossy().to_string();
        let cfg = config(
            get(&[("RESTLOOP_MODEL", "m"), ("RESTLOOP_SESSION", file.as_str())]),
            Instant::now(),
        )
        .unwrap();
        let e = conversation(&cfg).unwrap_err();
        assert_eq!(e, format!("{file} is not a restloop session"));
    }

    /// A session file that is not there is a new conversation with the system prompt.
    #[test]
    fn a_session_that_is_not_there_is_a_new_conversation() {
        // The directory of the test and the shell of the prompt come from the
        // environment: another test must not change them while this one runs, so
        // the shared env lock is held for the whole body.
        let _env = env_lock();
        // The file is not there: the path is of the directory of the test, so the
        // guard of the test cannot take a file that is not its own.
        let dir = TempDir::new("no-session");
        let file = dir.missing("session.json").to_string_lossy().to_string();
        let cfg = config(
            get(&[("RESTLOOP_MODEL", "m"), ("RESTLOOP_SESSION", file.as_str())]),
            Instant::now(),
        )
        .unwrap();
        let messages = conversation(&cfg).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(
            messages[0]["content"],
            restloop::system_prompt(&cfg, shell_name(), env::consts::OS, &cwd())
        );
    }

    #[test]
    fn a_file_that_is_not_a_session_is_no_session() {
        assert_eq!(parse_session("not a session"), None);
        assert_eq!(parse_session("{}"), None);
        // Another version, and a session without messages, are no sessions either.
        assert_eq!(
            parse_session(&json!({"version": 2, "messages": [{"role": "user"}]}).to_string()),
            None
        );
        assert_eq!(
            parse_session(&json!({"version": 1, "messages": []}).to_string()),
            None
        );
    }
}
