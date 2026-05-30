use oclapp_core::models::download::DownloadProgress;
use std::sync::atomic::{AtomicU64, Ordering};

#[test]
fn test_download_progress_callback() {
    let total = AtomicU64::new(0);
    let downloaded = AtomicU64::new(0);

    let cb = |progress: DownloadProgress| {
        total.store(progress.total_bytes, Ordering::SeqCst);
        downloaded.store(progress.downloaded_bytes, Ordering::SeqCst);
    };

    cb(DownloadProgress {
        total_bytes: 1000,
        downloaded_bytes: 500,
    });

    assert_eq!(total.load(Ordering::SeqCst), 1000);
    assert_eq!(downloaded.load(Ordering::SeqCst), 500);
}

#[test]
fn test_download_progress_clone() {
    let p1 = DownloadProgress {
        total_bytes: 1024,
        downloaded_bytes: 512,
    };
    let p2 = p1.clone();
    assert_eq!(p1.total_bytes, p2.total_bytes);
    assert_eq!(p1.downloaded_bytes, p2.downloaded_bytes);
}
