//! Unified cross-platform logging for Scallion Vocab.
//!
//! Uses the [`log`] crate as the logging facade.
//!
//! | Platform  | Output target          |
//! |-----------|------------------------|
//! | Android   | logcat (NDK FFI)       |
//! | Desktop   | stderr                 |
//!
//! # Initialisation
//!
//! Call [`init()`] **once** at program start:
//!
//! ```ignore
//! fn main() {
//!     logging::init();
//!     // …
//! }
//! ```
//!
//! # Log macro
//!
//! The convenience [`log!`] macro is an alias for [`log::info!`] so that
//! existing call sites keep working.  New code should prefer the standard
//! [`log::info!`], [`log::warn!`] etc. directly.
//!
//! ```ignore
//! log!("[Prefs::Load] preferences loaded");
//! ```
//!
//! # Runtime toggle
//!
//! ```ignore
//! logging::set_enabled(false);   // silence all output
//! logging::set_enabled(true);    // re-enable
//! ```
//!
//! # Log message format convention
//!
//! Every message MUST begin with a `[Component::Action]` prefix so that
//! logs are grep-friendly and self-documenting:
//!
//! | Prefix              | Where                         |
//! |---------------------|-------------------------------|
//! | `[Prefs::Load]`     | Loading stored preferences    |
//! | `[Prefs::Theme]`    | Theme persistence             |
//! | `[Prefs::SaveUrls]` | Saving recent URL history     |
//! | `[Upload::Fetch]`   | Fetching a deck               |
//! | `[Upload::Export]`  | Anki export flow              |
//! | `[Upload::Import]`  | Anki / HTML import            |
//! | `[Upload::HtmlFallback]` | Manual HTML paste fallback |
//! | `[Quiz::Keyboard]`  | Keyboard shortcut handling   |
//! | `[Quiz::AutoNext]`  | Auto-advance timer            |
//! | `[Quiz::Screen]`    | General quiz screen events    |
//! | `[App::Eval]`       | `document::eval()` calls      |

use std::sync::atomic::{AtomicBool, Ordering};

static ENABLED: AtomicBool = AtomicBool::new(true);

/// Globally enable or disable all log output.
///
/// Public API hook kept for runtime diagnostics — not referenced from within the crate.
#[allow(dead_code)]
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

/// Returns `true` if logging is currently enabled.
#[allow(dead_code)]
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

struct ScallionLogger;

impl log::Log for ScallionLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        ENABLED.load(Ordering::Relaxed)
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let msg = format!("{}", record.args());

        #[cfg(target_os = "android")]
        android_write(record.level(), &msg);

        #[cfg(not(target_os = "android"))]
        eprintln!("{msg}");
    }

    fn flush(&self) {}
}

static LOGGER: ScallionLogger = ScallionLogger;

/// Initialise exactly once, ideally first thing in `main()`. Subsequent calls short-circuit.
pub fn init() {
    // Debug builds get `Debug`-level visibility; release builds only `Info`.
    let max_level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    log::set_max_level(max_level);

    if log::set_logger(&LOGGER).is_err() {
        // Logger already registered (e.g. by a test runner) — benign.
    }
}

/// Convenience alias for [`log::info!`] — existing call sites keep working.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        ::log::info!($($arg)*)
    };
}

#[cfg(target_os = "android")]
fn android_write(level: log::Level, msg: &str) {
    use std::ffi::CString;

    const ANDROID_LOG_DEBUG: i32 = 3;
    const ANDROID_LOG_INFO: i32 = 4;
    const ANDROID_LOG_WARN: i32 = 5;
    const ANDROID_LOG_ERROR: i32 = 6;

    let prio = match level {
        log::Level::Error => ANDROID_LOG_ERROR,
        log::Level::Warn => ANDROID_LOG_WARN,
        log::Level::Info => ANDROID_LOG_INFO,
        log::Level::Debug | log::Level::Trace => ANDROID_LOG_DEBUG,
    };

    let tag = CString::new("ScallionVocab").unwrap_or_default();
    let text = CString::new(msg).unwrap_or_default();

    unsafe extern "C" {
        fn __android_log_write(
            prio: i32,
            tag: *const std::ffi::c_char,
            text: *const std::ffi::c_char,
        ) -> i32;
    }

    unsafe {
        __android_log_write(prio, tag.as_ptr(), text.as_ptr());
    }
}
