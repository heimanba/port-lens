//! UI strings for tables and banners (single locale for now; extend `UiLang` when needed).

use crate::commands::common::ListenerStatus;

#[derive(Clone, Copy, Debug)]
pub enum UiLang {
    En,
}

impl UiLang {
    pub fn current() -> Self {
        Self::En
    }
}

pub fn banner_title(_lang: UiLang) -> &'static str {
    "Port lens"
}

pub fn banner_subtitle(_lang: UiLang) -> &'static str {
    "listening to your ports..."
}

pub fn port_table_headers(_lang: UiLang) -> [&'static str; 7] {
    [
        "PORT",
        "PROCESS",
        "PID",
        "PROJECT",
        "FRAMEWORK",
        "UPTIME",
        "STATUS",
    ]
}

pub fn ps_table_headers(_lang: UiLang) -> [&'static str; 8] {
    [
        "PID",
        "PROCESS",
        "CPU%",
        "MEM",
        "PROJECT",
        "FRAMEWORK",
        "UPTIME",
        "WHAT",
    ]
}

/// Plain status text for the port table (ANSI color applied separately in `display::table`).
pub fn listener_status_label(s: ListenerStatus, _lang: UiLang) -> &'static str {
    match s {
        ListenerStatus::Healthy => "● healthy",
        ListenerStatus::Orphaned => "● orphaned",
        ListenerStatus::Zombie => "● zombie",
    }
}
