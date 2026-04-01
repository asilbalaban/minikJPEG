use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

/// Aktif batch işinin durumu.
#[derive(Debug)]
pub struct JobState {
    pub cancel_flag: Arc<AtomicBool>,
    #[allow(dead_code)]
    pub total_files: usize,
    #[allow(dead_code)]
    pub completed_files: usize,
}

/// Uygulama genelinde paylaşılan durum.
#[derive(Default)]
pub struct AppState {
    /// Aktif batch işleri: job_id → JobState
    pub jobs: Mutex<HashMap<String, JobState>>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}
