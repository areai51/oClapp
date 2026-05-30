use oclapp_core::server::binary::{detect_platform, find_binary, well_known_dirs, BinaryFlavor};
use std::fs::{self, File};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_detect_platform_returns_url_and_name() {
    let (name, url) = detect_platform();
    assert!(name == "llama-server" || name == "llama-server.exe");
    assert!(url.contains("llama-b4744-bin-"));
    assert!(url.starts_with("https://github.com/ggml-org/llama.cpp/releases/latest/download/"));
}

#[test]
fn test_well_known_dirs_includes_common_paths() {
    let dirs = well_known_dirs();
    let paths: Vec<&PathBuf> = dirs.iter().collect();

    if cfg!(target_os = "macos") {
        assert!(paths.contains(&&PathBuf::from("/opt/homebrew/bin")));
        assert!(paths.contains(&&PathBuf::from("/usr/local/bin")));
    }
    if cfg!(target_os = "linux") {
        assert!(paths.contains(&&PathBuf::from("/usr/local/bin")));
        assert!(paths.contains(&&PathBuf::from("/usr/bin")));
    }

    // All entries should include home dirs if HOME is set
    if std::env::var("HOME").is_ok() {
        assert!(paths.iter().any(|p: &&PathBuf| p.to_string_lossy().contains(".local/bin")));
    }
}

#[test]
fn test_find_binary_in_well_known_dir() {
    let dir = TempDir::new().unwrap();
    let fake_bin = dir.path().join("llama");
    File::create(&fake_bin).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&fake_bin).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake_bin, perms).unwrap();
    }

    // Temporarily inject our fake dir into well-known dirs via PATH
    let old_path = std::env::var_os("PATH");
    let new_path = if let Some(ref old) = old_path {
        let mut s = old.to_string_lossy().to_string();
        s.push(':');
        s.push_str(dir.path().to_str().unwrap());
        s
    } else {
        dir.path().to_string_lossy().to_string()
    };
    std::env::set_var("PATH", &new_path);

    let found = find_binary("llama");
    assert!(found.is_some());
    assert_eq!(found.unwrap(), fake_bin);

    if let Some(old) = old_path {
        std::env::set_var("PATH", old);
    } else {
        std::env::remove_var("PATH");
    }
}

#[test]
fn test_find_binary_llama_server() {
    let dir = TempDir::new().unwrap();
    let fake_bin = dir.path().join("llama-server");
    File::create(&fake_bin).unwrap();

    let old_path = std::env::var_os("PATH");
    let new_path = if let Some(ref old) = old_path {
        let mut s = old.to_string_lossy().to_string();
        s.push(':');
        s.push_str(dir.path().to_str().unwrap());
        s
    } else {
        dir.path().to_string_lossy().to_string()
    };
    std::env::set_var("PATH", &new_path);

    let found = find_binary("llama-server");
    assert!(found.is_some());

    if let Some(old) = old_path {
        std::env::set_var("PATH", old);
    } else {
        std::env::remove_var("PATH");
    }
}

#[test]
fn test_find_binary_missing() {
    let dir = TempDir::new().unwrap();
    let old_path = std::env::var_os("PATH");
    std::env::set_var("PATH", dir.path().to_str().unwrap());

    let found = find_binary("definitely-not-real-12345");
    assert!(found.is_none());

    if let Some(old) = old_path {
        std::env::set_var("PATH", old);
    } else {
        std::env::remove_var("PATH");
    }
}

#[test]
fn test_binary_flavor_equality() {
    assert_eq!(BinaryFlavor::LlamaApp, BinaryFlavor::LlamaApp);
    assert_eq!(BinaryFlavor::Legacy, BinaryFlavor::Legacy);
    assert_ne!(BinaryFlavor::LlamaApp, BinaryFlavor::Legacy);
}
