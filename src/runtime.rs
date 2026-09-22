//! Process-wide runtime setup: the rayon thread pool and the logger.

use log::LevelFilter;
use std::io::Write;

/// Upper bound on the global rayon pool — see [`init_thread_pool`]. Past a
/// handful of threads the nesting overhead grows faster than the work shrinks,
/// and a machine with fewer cores than this should not be oversubscribed.
const MAX_RAYON_THREADS: usize = 8;

/// Sizes the global rayon pool once, before any parsing touches it.
///
/// gitronics parallelises across *files* on the same pool migjorn parallelises
/// within a file. Nesting the two on a pool sized to the core count spends most
/// of its time parking and waking threads rather than doing work: on a 96-core
/// machine, assembling a 376 MB model costs 12.1 s of user time against 17.3 s
/// of system time, which capping the pool brings down to 7.3 s and 3.8 s.
/// migjorn's own module documentation names this case and prescribes the cap.
///
/// This does not affect the output. migjorn's segmentation is independent of
/// the pool size, so a build depends only on its inputs — `tests/test_determinism.rs`
/// pins that.
///
/// An explicit `RAYON_NUM_THREADS` is the user's decision and is left alone.
/// Failure to build the pool means one already exists — nothing we need to act
/// on.
pub fn init_thread_pool() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        if std::env::var_os("RAYON_NUM_THREADS").is_some() {
            return;
        }
        let threads = std::thread::available_parallelism()
            .map_or(MAX_RAYON_THREADS, |p| p.get().min(MAX_RAYON_THREADS));
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global();
    });
}

/// Initializes the application logger with custom formatting.
///
/// Sets up env_logger with INFO level by default (can be overridden by RUST_LOG).
/// Log format: `[YYYY-MM-DD HH:MM:SS LEVEL target] message`
pub fn init_logger() {
    let mut logger = env_logger::Builder::new();

    // Default to INFO from code, but still allow RUST_LOG to override it.
    logger.filter_level(LevelFilter::Info);
    logger.parse_default_env();

    // Custom format: [2026-05-20 09:27:04 INFO gitronics_rs::build_model] message
    logger.format(|buf, record| {
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(
            buf,
            "[{} {} gitronics] {}",
            ts,
            record.level(),
            record.args()
        )
    });
    logger.try_init().ok();
}
