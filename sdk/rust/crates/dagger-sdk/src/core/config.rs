use crate::core::logger::DynLogger;
use derive_builder::Builder;
use std::path::PathBuf;

#[derive(Default, Builder)]
#[builder(build_fn(private, name = "fallible_build"))]
#[builder(setter(strip_option))]
pub struct Config {
    #[builder(default = "None")]
    /// The host workdir loaded into dagger.
    pub workdir_path: Option<PathBuf>,
    #[builder(default = "None")]
    /// Project configuration file path.
    pub config_path: Option<PathBuf>,
    #[builder(default = "10 * 1000")]
    /// The maximum time in milliseconds for establishing a connection to the server.
    /// Defaults to 10 seconds.
    pub timeout_ms: u64,
    #[builder(default = "None")]
    /// The maximum time in milliseconds for executing a request.
    /// Defaults to no timeout.
    pub execute_timeout_ms: Option<u64>,
    #[builder(default = "None")]
    /// Logger implementation to handle logs from the engine.
    pub logger: Option<DynLogger>,
    /// Spawn a terminal on container exec failure
    #[builder(default = "false")]
    pub interactive: bool,
    /// Show the Dagger TUI (requires stderr to be a TTY).
    /// When false (the default), stderr is piped and either forwarded to the
    /// logger or discarded, matching the behavior of other Dagger SDKs.
    #[builder(default = "false")]
    pub tui: bool,
}

impl ConfigBuilder {
    pub fn build(&mut self) -> Config {
        self.fallible_build()
            .expect("all fields have default values")
    }
}

impl Config {
    pub fn new(
        workdir_path: Option<PathBuf>,
        config_path: Option<PathBuf>,
        timeout_ms: Option<u64>,
        execute_timeout_ms: Option<u64>,
        logger: Option<DynLogger>,
        interactive: bool,
    ) -> Self {
        Self {
            workdir_path,
            config_path,
            timeout_ms: timeout_ms.unwrap_or(10 * 1000),
            execute_timeout_ms,
            logger,
            interactive,
            tui: false,
        }
    }

    /// Returns a new config builder instance
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}
