use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use cunny_lang::{Machine, brat_correction, build_site, compile};

const BAD_APPLE_FPS: u64 = 15;
const FRAME_HOME: &[u8] = b"\x1b[f";

pub fn run() -> Result<(), String> {
    let arguments: Vec<String> = env::args().collect();
    let program = arguments.first().map(String::as_str).unwrap_or("cunny");

    match arguments.as_slice() {
        [_, flag] if flag == "--version" || flag == "-V" => {
            println!("cunny {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        [_, flag] if flag == "--help" || flag == "-h" => {
            println!("{}", usage(program));
            Ok(())
        }
        [_, first, second, root] if first == "brat" && second == "correction" => {
            brat_correction(Path::new(root))
        }
        [_, command, root] if command == "build" => build(root),
        [_, flag, path] if flag == "--imouto" => run_file(path, true),
        [_, path] if path != "build" && path != "brat" && path != "--imouto" => {
            run_file(path, false)
        }
        _ => Err(usage(program)),
    }
}

fn run_file(path: &str, imouto: bool) -> Result<(), String> {
    if !path.ends_with(".cunny") {
        return Err("programs must use the .cunny extension".to_owned());
    }

    let source =
        fs::read_to_string(path).map_err(|error| format!("could not read '{path}': {error}"))?;
    let instructions = compile(&source).map_err(|error| {
        if imouto {
            format!("imouto found a problem: {error}")
        } else {
            error
        }
    })?;

    let mut machine = Machine::new();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = stdin.lock();

    if imouto {
        let mut output = stdout.lock();
        let stderr = io::stderr();
        let mut trace = stderr.lock();
        machine
            .run_with_imouto(&instructions, &mut input, &mut output, &mut trace)
            .map_err(|error| format!("imouto stopped: {error}"))?;
        output
            .flush()
            .map_err(|error| format!("could not flush output: {error}"))?;
    } else if is_bad_apple_demo(Path::new(path)) {
        let audio_path = Path::new(path).with_extension("m4a");
        let _audio = match AudioPlayer::start(&audio_path) {
            Ok(player) => Some(player),
            Err(error) => {
                eprintln!("warning: {error}");
                None
            }
        };
        let mut output = FramePacedWriter::new(stdout.lock(), BAD_APPLE_FPS);
        machine.run(&instructions, &mut input, &mut output)?;
        output
            .flush()
            .map_err(|error| format!("could not flush output: {error}"))?;
    } else {
        let mut output = stdout.lock();
        machine.run(&instructions, &mut input, &mut output)?;
        output
            .flush()
            .map_err(|error| format!("could not flush output: {error}"))?;
    }

    Ok(())
}

fn is_bad_apple_demo(path: &Path) -> bool {
    path.file_stem().and_then(|stem| stem.to_str()) == Some("bad-apple")
}

struct FramePacedWriter<W: Write> {
    inner: W,
    pending: Vec<u8>,
    started_at: Option<Instant>,
    frame_markers: u64,
    frames_per_second: u64,
}

impl<W: Write> FramePacedWriter<W> {
    fn new(inner: W, frames_per_second: u64) -> Self {
        Self {
            inner,
            pending: Vec::with_capacity(FRAME_HOME.len()),
            started_at: None,
            frame_markers: 0,
            frames_per_second,
        }
    }

    fn write_byte(&mut self, byte: u8) -> io::Result<()> {
        if self.pending.is_empty() && byte != FRAME_HOME[0] {
            return self.inner.write_all(&[byte]);
        }

        self.pending.push(byte);
        if FRAME_HOME.starts_with(&self.pending) {
            if self.pending.len() == FRAME_HOME.len() {
                self.write_frame_marker()?;
                self.pending.clear();
            }
            return Ok(());
        }

        self.inner.write_all(&self.pending)?;
        self.pending.clear();
        Ok(())
    }

    fn write_frame_marker(&mut self) -> io::Result<()> {
        if let Some(started_at) = self.started_at {
            self.inner.flush()?;
            let target =
                Duration::from_secs_f64(self.frame_markers as f64 / self.frames_per_second as f64);
            thread::sleep(target.saturating_sub(started_at.elapsed()));
        } else {
            self.started_at = Some(Instant::now());
        }

        self.frame_markers += 1;
        self.inner.write_all(FRAME_HOME)
    }
}

impl<W: Write> Write for FramePacedWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        for &byte in buffer {
            self.write_byte(byte)?;
        }
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.pending.is_empty() {
            self.inner.write_all(&self.pending)?;
            self.pending.clear();
        }
        self.inner.flush()
    }
}

struct AudioPlayer {
    child: Child,
}

impl AudioPlayer {
    fn start(path: &Path) -> Result<Self, String> {
        if !path.is_file() {
            return Err(format!(
                "audio file '{}' is missing; playing the animation silently",
                path.display()
            ));
        }

        spawn_audio(path)
            .map(|child| Self { child })
            .map_err(|error| {
                format!(
                    "could not start audio from '{}': {error}; playing the animation silently",
                    path.display()
                )
            })
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn quiet_command(program: &str) -> Command {
    let mut command = Command::new(program);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

#[cfg(target_os = "macos")]
fn spawn_audio(path: &Path) -> io::Result<Child> {
    quiet_command("afplay").arg(path).spawn()
}

#[cfg(target_os = "linux")]
fn spawn_audio(path: &Path) -> io::Result<Child> {
    let players: [(&str, &[&str]); 3] = [
        ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"]),
        ("mpv", &["--no-video", "--really-quiet"]),
        ("cvlc", &["--intf", "dummy", "--play-and-exit"]),
    ];
    let mut last_error = None;

    for (program, arguments) in players {
        match quiet_command(program).args(arguments).arg(path).spawn() {
            Ok(child) => return Ok(child),
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "no supported audio player found")
    }))
}

#[cfg(target_os = "windows")]
fn spawn_audio(path: &Path) -> io::Result<Child> {
    const SCRIPT: &str = "Add-Type -AssemblyName presentationCore; \
        $player = New-Object system.windows.media.mediaplayer; \
        $player.open([uri]$env:CUNNY_AUDIO_PATH); \
        $player.Play(); Start-Sleep -Seconds 220";

    quiet_command("powershell")
        .args(["-NoProfile", "-Command", SCRIPT])
        .env("CUNNY_AUDIO_PATH", path)
        .spawn()
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn spawn_audio(_path: &Path) -> io::Result<Child> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "audio playback is not supported on this platform",
    ))
}

fn build(root: &str) -> Result<(), String> {
    let report = build_site(Path::new(root))?;
    println!(
        "Built {} page(s) in {}",
        report.pages,
        report.output.display()
    );
    Ok(())
}

fn usage(program: &str) -> String {
    format!(
        "usage:\n  {program} <file.cunny>\n  {program} --imouto <file.cunny>\n  {program} build <site-directory>\n  {program} brat correction <site-directory>\n  {program} --help\n  {program} --version"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_pacer_preserves_output_and_finds_markers() {
        let source = b"\x1b[?25l\x1b[2J\x1b[fframe one\x1b[fframe two";
        let mut writer = FramePacedWriter::new(Vec::new(), 1_000_000_000);

        writer.write_all(source).unwrap();
        writer.flush().unwrap();

        assert_eq!(writer.inner, source);
        assert_eq!(writer.frame_markers, 2);
    }

    #[test]
    fn only_the_named_example_uses_demo_playback() {
        assert!(is_bad_apple_demo(Path::new("examples/bad-apple.cunny")));
        assert!(!is_bad_apple_demo(Path::new("examples/helloworld.cunny")));
    }
}
