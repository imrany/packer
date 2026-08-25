use log::{error, info, warn};
use std::env::consts::{ARCH, OS};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// A running Cloudflare quick tunnel. Killing/dropping this struct
/// terminates the underlying `cloudflared` process.
pub struct Tunnel {
    child: Option<Child>,
    pub public_url: String,
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            if let Err(e) = child.kill() {
                warn!("Could not stop cloudflared process cleanly: {}", e);
            }
            // Delegate the blocking wait call to a standard OS thread
            // to avoid blocking Tokio/Actix's async runtime context.
            thread::spawn(move || {
                let _ = child.wait();
            });
        }
    }
}

/// Filename the bundled binary should have next to the packer executable.
fn bundled_binary_name() -> &'static str {
    if cfg!(windows) {
        "cloudflared.exe"
    } else {
        "cloudflared"
    }
}

/// Where packer keeps its private copy of cloudflared (next to the packer executable).
fn bundled_binary_path(app_dir: &Path) -> PathBuf {
    app_dir.join(bundled_binary_name())
}

/// Returns true if the binary at `path` exists and actually runs.
fn binary_works(path: &Path) -> bool {
    Command::new(path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Checks if `cloudflared` is available globally on system PATH without external dependencies.
fn system_cloudflared_exists() -> Option<PathBuf> {
    let binary_name = bundled_binary_name();
    
    // Test if `cloudflared` is directly executable via system PATH
    if binary_works(Path::new(binary_name)) {
        return Some(PathBuf::from(binary_name));
    }

    None
}

/// The official release asset name for the running platform.
fn release_asset_name() -> Result<&'static str, String> {
    match (OS, ARCH) {
        ("linux", "x86_64") => Ok("cloudflared-linux-amd64"),
        ("linux", "aarch64") => Ok("cloudflared-linux-arm64"),
        ("linux", "arm") => Ok("cloudflared-linux-arm"),
        ("linux", "x86") => Ok("cloudflared-linux-386"),
        ("macos", "x86_64") => Ok("cloudflared-darwin-amd64.tgz"),
        ("macos", "aarch64") => Ok("cloudflared-darwin-arm64.tgz"),
        ("windows", "x86_64") => Ok("cloudflared-windows-amd64.exe"),
        ("windows", "x86") => Ok("cloudflared-windows-386.exe"),
        (os, arch) => Err(format!(
            "No bundled cloudflared build is available for {os}/{arch}"
        )),
    }
}

/// Ensures a working `cloudflared` binary is present.
/// Checks system PATH first, then local directory, and only downloads as a last resort.
pub fn ensure_cloudflared(app_dir: &Path) -> Result<PathBuf, String> {
    // 1. First priority: Check if `cloudflared` is installed globally on the system PATH
    if let Some(sys_path) = system_cloudflared_exists() {
        info!("Found existing system cloudflared on PATH");
        return Ok(sys_path);
    }

    // 2. Second priority: Check if a working copy exists in `app_dir`
    let dest = bundled_binary_path(app_dir);
    if dest.exists() && binary_works(&dest) {
        return Ok(dest);
    }

    // 3. Last resort: Download only if not found anywhere else
    let app_dir = app_dir.to_path_buf();
    thread::spawn(move || ensure_cloudflared_internal(&app_dir))
        .join()
        .map_err(|_| "Failed to join cloudflared download thread".to_string())?
}

fn ensure_cloudflared_internal(app_dir: &Path) -> Result<PathBuf, String> {
    let dest = bundled_binary_path(app_dir);
    let asset_name = release_asset_name()?;
    let url = format!("https://github.com/cloudflare/cloudflared/releases/latest/download/{asset_name}");

    info!("cloudflared not found locally. Setting up cloudflared (one-time download, ~40MB)...");

    let tmp_download = app_dir.join(format!("{asset_name}.download"));
    {
        let mut file = fs::File::create(&tmp_download)
            .map_err(|e| format!("Could not create temp file for cloudflared download: {e}"))?;
        let mut download = self_update::Download::from_url(&url);
        download.show_progress(true);
        download
            .download_to(&mut file)
            .map_err(|e| format!("Failed to download cloudflared: {e}"))?;
    }

    if asset_name.ends_with(".tgz") {
        extract_from_tgz(&tmp_download, &dest)?;
        let _ = fs::remove_file(&tmp_download);
    } else {
        fs::rename(&tmp_download, &dest)
            .map_err(|e| format!("Failed to install cloudflared binary: {e}"))?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&dest)
            .map_err(|e| format!("Failed to read cloudflared permissions: {e}"))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&dest, perms)
            .map_err(|e| format!("Failed to make cloudflared executable: {e}"))?;
    }

    if !binary_works(&dest) {
        return Err("Downloaded cloudflared binary does not appear to work".to_string());
    }

    info!("cloudflared is ready at {:?}", dest);
    Ok(dest)
}

fn extract_from_tgz(tgz_path: &Path, dest: &Path) -> Result<(), String> {
    let extract_dir = tgz_path.with_extension("extracted");
    fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("Failed to create extraction dir: {e}"))?;

    let status = Command::new("tar")
        .arg("-xzf")
        .arg(tgz_path)
        .arg("-C")
        .arg(&extract_dir)
        .status()
        .map_err(|e| format!("Failed to run tar to extract cloudflared: {e}"))?;

    if !status.success() {
        return Err("tar failed to extract the cloudflared archive".to_string());
    }

    let extracted_binary = extract_dir.join("cloudflared");
    fs::rename(&extracted_binary, dest)
        .map_err(|e| format!("Failed to move extracted cloudflared binary: {e}"))?;

    let _ = fs::remove_dir_all(&extract_dir);
    Ok(())
}

pub fn print_install_instructions() {
    warn!("Could not set up the bundled cloudflared binary automatically.");
    match OS {
        "macos" => info!("You can install it yourself with: brew install cloudflared"),
        "linux" => info!(
            "You can install it yourself with, e.g.:\n  \
             curl -L -o cloudflared.deb https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64.deb\n  \
             sudo dpkg -i cloudflared.deb"
        ),
        "windows" => {
            info!("You can install it yourself with: winget install --id Cloudflare.cloudflared")
        }
        other => info!(
            "Install instructions for {}: https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/",
            other
        ),
    }
}

pub fn start_cloudflare_tunnel(cloudflared_path: &Path, port: u16) -> Result<Tunnel, String> {
    let mut child = Command::new(cloudflared_path)
        .args([
            "tunnel",
            "--url",
            &format!("http://localhost:{}", port),
            "--no-autoupdate",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn cloudflared: {}", e))?;

    let (tx, rx) = mpsc::channel::<String>();

    if let Some(stderr) = child.stderr.take() {
        let tx = tx.clone();
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().flatten() {
                if let Some(url) = extract_trycloudflare_url(&line) {
                    let _ = tx.send(url);
                }
            }
        });
    }

    if let Some(stdout) = child.stdout.take() {
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().flatten() {
                if let Some(url) = extract_trycloudflare_url(&line) {
                    let _ = tx.send(url);
                }
            }
        });
    }

    match rx.recv_timeout(Duration::from_secs(20)) {
        Ok(public_url) => Ok(Tunnel {
            child: Some(child),
            public_url,
        }),
        Err(_) => {
            let _ = child.kill();
            error!("Timed out waiting for cloudflared to report a public URL.");
            Err("cloudflared did not establish a tunnel within 20 seconds".to_string())
        }
    }
}

fn extract_trycloudflare_url(line: &str) -> Option<String> {
    let start = line.find("https://")?;
    let candidate = &line[start..];
    let end = candidate
        .find(|c: char| c.is_whitespace() || c == '|')
        .unwrap_or(candidate.len());
    let url = &candidate[..end];
    if url.contains("trycloudflare.com") {
        Some(url.to_string())
    } else {
        None
    }
}
