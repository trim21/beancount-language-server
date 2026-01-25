use clap::{Command, arg};
use std::fs;
use std::io;
use std::sync::Arc;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::{EnvFilter, filter::Directive};

fn main() {
    let matches = Command::new("beancount-language-server")
        .args(&[
            arg!(--stdio "specifies to use stdio to communicate with lsp"),
            arg!(--log [LOG_LEVEL] "write log to default file beancount-language-server.log with optional level (deprecated, use --log-file and --log-level)"),
            arg!(--"log-file" <LOG_FILE> "write log output to the specified file instead of stderr"),
            arg!(--"log-level" [LOG_LEVEL] "set log level (trace, debug, info, warn, error, off); defaults to info"),
            arg!(version: -v --version),
        ])
        .get_matches();

    if matches.args_present() && matches.get_flag("version") {
        print!("{}", std::env!("CARGO_PKG_VERSION"));
        return;
    }

    let log_file = matches.get_one::<String>("log-file").cloned().or_else(|| {
        if matches.contains_id("log") {
            Some("beancount-language-server.log".to_owned())
        } else {
            None
        }
    });

    let log_level = matches
        .get_one::<String>("log-level")
        .or_else(|| matches.get_one::<String>("log"));

    setup_logging(log_file.as_deref(), log_level);

    tracing::info!(
        "Starting beancount-language-server v{}",
        env!("CARGO_PKG_VERSION")
    );
    tracing::debug!(
        "Command line args: stdio={}, log_to_file={}, log_level={:?}",
        matches.get_flag("stdio"),
        log_file.unwrap_or("stderr".to_string()),
        log_level
    );

    match beancount_language_server::run_server() {
        Ok(()) => {
            tracing::info!("Language server shutdown gracefully");
        }
        Err(e) => {
            tracing::error!("Language server failed with error: {}", e);
            std::process::exit(1);
        }
    }
}

fn setup_logging(log_file: Option<&str>, log_level_arg: Option<&String>) {
    let level = match log_level_arg {
        Some(level_str) => parse_log_level(level_str),
        None => LevelFilter::INFO, // Default level when not specified
    };

    let file = match log_file {
        Some(path) => match fs::OpenOptions::new().create(true).append(true).open(path) {
            Ok(f) => {
                eprintln!("Logging to file: {path}");
                Some(f)
            }
            Err(e) => {
                eprintln!("Failed to open log file '{path}': {e}. Falling back to stderr.");
                None
            }
        },
        None => None,
    };

    let writer = match file {
        Some(file) => BoxMakeWriter::new(Arc::new(file)),
        None => BoxMakeWriter::new(io::stderr),
    };

    let filter = EnvFilter::default().add_directive(Directive::from(level));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();
}

fn parse_log_level(level_str: &str) -> LevelFilter {
    match level_str.to_lowercase().as_str() {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        "off" => LevelFilter::OFF,
        _ => {
            eprintln!(
                "Invalid log level '{level_str}'. Using 'info' as default. Valid levels: trace, debug, info, warn, error, off"
            );
            LevelFilter::INFO
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_level_valid_lowercase() {
        assert_eq!(parse_log_level("trace"), LevelFilter::TRACE);
        assert_eq!(parse_log_level("debug"), LevelFilter::DEBUG);
        assert_eq!(parse_log_level("info"), LevelFilter::INFO);
        assert_eq!(parse_log_level("warn"), LevelFilter::WARN);
        assert_eq!(parse_log_level("error"), LevelFilter::ERROR);
        assert_eq!(parse_log_level("off"), LevelFilter::OFF);
    }

    #[test]
    fn test_parse_log_level_valid_uppercase() {
        assert_eq!(parse_log_level("TRACE"), LevelFilter::TRACE);
        assert_eq!(parse_log_level("DEBUG"), LevelFilter::DEBUG);
        assert_eq!(parse_log_level("INFO"), LevelFilter::INFO);
        assert_eq!(parse_log_level("WARN"), LevelFilter::WARN);
        assert_eq!(parse_log_level("ERROR"), LevelFilter::ERROR);
        assert_eq!(parse_log_level("OFF"), LevelFilter::OFF);
    }

    #[test]
    fn test_parse_log_level_valid_mixed_case() {
        assert_eq!(parse_log_level("Trace"), LevelFilter::TRACE);
        assert_eq!(parse_log_level("Debug"), LevelFilter::DEBUG);
        assert_eq!(parse_log_level("Info"), LevelFilter::INFO);
        assert_eq!(parse_log_level("Warn"), LevelFilter::WARN);
        assert_eq!(parse_log_level("Error"), LevelFilter::ERROR);
        assert_eq!(parse_log_level("Off"), LevelFilter::OFF);
    }

    #[test]
    fn test_parse_log_level_invalid_defaults_to_info() {
        assert_eq!(parse_log_level("invalid"), LevelFilter::INFO);
        assert_eq!(parse_log_level("unknown"), LevelFilter::INFO);
        assert_eq!(parse_log_level(""), LevelFilter::INFO);
        assert_eq!(parse_log_level("123"), LevelFilter::INFO);
    }
}
