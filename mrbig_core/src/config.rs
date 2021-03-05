use serde_derive::Deserialize;
use std::io::Read;

const PORT: &str = "port";
const CONFIG: &str = "config";
const HOSTNAME: &str = "hostname";
const DEBUG: &str = "debug";

/// gRPC server related configuration parameters.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct GrpcServer {
    pub concurrency_limit_per_connection: Option<usize>,
    pub timeout: Option<std::time::Duration>,
    pub tcp_keepalive: Option<std::time::Duration>,
    #[serde(default = "default_grpc_reflection")]
    pub reflection: bool,
}

/// `Mr. Big` service specific configuration parameters.
#[derive(Clone, Debug, Deserialize)]
pub struct Service {
    /// Port number to bind when serving.
    #[serde(default = "default_port")]
    pub port: u16,
    /// Hostname to bind to when serving.
    #[serde(default = "default_hostname")]
    pub hostname: String,
    /// Enable debug mode which prints verbose messages.
    #[serde(default)]
    pub debug: bool,
    /// Enable trace mode which prints very verbose messages.
    #[serde(default)]
    pub trace: bool,
    /// Logger filters as by env_logger format.
    #[serde(default = "default_logger_filters")]
    pub logger_filters: String,
    /// gRPC server related configuration.
    #[serde(default)]
    pub grpc_server: GrpcServer,
    /// Metrics related configurations.
    #[cfg(feature = "telemetry")]
    #[serde(default)]
    pub metrics: crate::metrics::Config,
}

/// Config struct used to deserialize all the configuration parameters.
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    /// `Mr. Big` specific configuration parameters.
    #[serde(default = "default_service")]
    pub service: Service,
    #[serde(default)]
    raw: toml::value::Table,
}

fn default_service() -> Service {
    toml::from_str("").unwrap()
}

fn default_port() -> u16 {
    3000
}
fn default_hostname() -> String {
    "0.0.0.0".into()
}

fn default_logger_filters() -> String {
    "h2=warn,hyper=warn,tower_buffer=warn".into()
}

fn default_grpc_reflection() -> bool {
    true
}

fn usage(program: &str, opts: getopts::Options) -> String {
    let brief = format!("Usage: {} [options]", program);
    opts.usage(&brief)
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str("").unwrap()
    }
}

impl Config {
    /// Read configuration values from the command line arguments and parse
    /// a config TOML file if available. Fails if command line arguments are used
    /// improperly, or if config file is not found or not parseable.
    pub fn from_args() -> std::result::Result<Config, crate::error::Error> {
        Config::from_args_vec(std::env::args().collect())
    }

    /// Try to deserialize into type `T` the configuration values which are not
    /// `Mr. Big` specific. Type `T` must implement `serde::Deserialize`.
    pub fn try_raw_into<'de, T>(&mut self) -> std::result::Result<T, toml::de::Error>
    where
        T: serde::de::Deserialize<'de>,
    {
        // consume the raw TOML table
        let value = toml::Value::Table(std::mem::take(&mut self.raw));
        value.try_into()
    }

    fn from_bytes(buffer: Vec<u8>) -> std::result::Result<Config, crate::error::Error> {
        let mut raw: toml::value::Table = toml::de::from_slice(&buffer)?;

        let service: Service = match raw.remove("service") {
            Some(srv) => srv.try_into()?,
            None => toml::from_str("")?,
        };

        Ok(Config { service, raw })
    }

    fn from_file(path: &str) -> std::result::Result<Config, crate::error::Error> {
        let mut f = std::fs::File::open(path)?;
        let mut buffer: Vec<u8> = Vec::new();

        f.read_to_end(&mut buffer)?;

        Config::from_bytes(buffer)
    }

    pub fn from_args_vec(args: Vec<String>) -> std::result::Result<Config, crate::error::Error> {
        let mut args = args;
        let program = args.remove(0);

        let mut opts = getopts::Options::new();
        opts.optopt("c", CONFIG, "set TOML config file name", "NAME");
        opts.optopt("p", PORT, "set port to bind server", "PORT");
        opts.optopt("", HOSTNAME, "set hostname to bind server", "HOSTNAME");
        opts.optflagmulti("d", DEBUG, "enable debug");
        opts.optflag("h", "help", "print this help menu");
        let matches = opts.parse(&args[..])?;

        if matches.opt_present("h") {
            panic!(usage(&program, opts));
        }

        // Command line flags override TOML
        let mut cfg_toml: Config = match matches.opt_str(CONFIG) {
            Some(path) => Config::from_file(&path)?,
            None => toml::from_str("")?, // create a default config
        };

        let mut srv = &mut cfg_toml.service;

        if let Some(port) = matches.opt_str(PORT) {
            srv.port = port
                .parse()
                .map_err(|e: std::num::ParseIntError| e.to_string())?;
        }

        if let Some(hostname) = matches.opt_str(HOSTNAME) {
            srv.hostname = hostname;
        }

        if matches.opt_present(DEBUG) {
            srv.debug = true;
            if matches.opt_count(DEBUG) > 1 {
                srv.trace = true;
            }
        }

        Ok(cfg_toml)
    }
}

/// Empty struct to be used as the extra placeholder in Configurable trait.
#[derive(Deserialize)]
pub struct Void {}

pub trait Configurable<'de> {
    fn get_config(&self) -> Option<&Config> {
        None
    }
    fn set_config(&mut self, _config: Config) {}

    type Extra: serde::de::Deserialize<'de>;
    fn set_config_extra(&mut self, _extra: Self::Extra) {}

    /// Allow taking the config itself (or a clone of it).
    /// Default implementation returns a clone.
    fn take_config(&mut self) -> Option<Config> {
        self.get_config().cloned()
    }

    /// Loads the configuration from a Vec of strings.
    /// Trait definition implements default behavior.
    /// Should not have to be re-implemented.
    fn load_from_args_vec(&mut self, args: Vec<String>) -> Result<(), crate::error::Error> {
        let mut config = Config::from_args_vec(args)?;
        self.set_config_extra(config.try_raw_into()?);
        self.set_config(config);
        Ok(())
    }

    /// Loads the configuration from command line args and TOML file if applicable.
    /// Trait definition implements default behavior.
    /// Should not have to be re-implemented.
    fn load_from_args(&mut self) -> Result<(), crate::error::Error> {
        self.load_from_args_vec(std::env::args().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let service: Service = toml::from_str("").unwrap();

        assert_eq!(service.port, default_port());
        assert_eq!(service.hostname, default_hostname());
    }

    #[test]
    fn just_args() {
        let cfg = Config::from_args_vec(vec![
            "mrbig".into(),
            "--port".into(),
            "4444".into(),
            "--hostname".into(),
            "localhost".into(),
            "--debug".into(),
        ])
        .unwrap();

        assert_eq!(cfg.service.port, 4444);
        assert_eq!(cfg.service.hostname, "localhost");
        assert_eq!(cfg.service.debug, true);

        let empty_grpc_server: GrpcServer = toml::from_str("").unwrap();
        assert_eq!(
            empty_grpc_server.concurrency_limit_per_connection,
            cfg.service.grpc_server.concurrency_limit_per_connection
        );
        assert_eq!(
            empty_grpc_server.tcp_keepalive,
            cfg.service.grpc_server.tcp_keepalive
        );
        assert_eq!(empty_grpc_server.timeout, cfg.service.grpc_server.timeout);
    }

    #[test]
    fn toml_file() {
        let contents: &str = r#"
        my_port = 39999
        [service]
        port = 8080
        hostname = "localhost"
        debug = true
        [service.grpc_server.timeout]
        secs = 2
        nanos = 0
        "#;

        #[derive(serde_derive::Serialize)]
        struct Foo {
            dur: std::time::Duration,
        }

        let mut cfg = Config::from_bytes(contents.as_bytes().to_vec()).unwrap();

        #[derive(Deserialize)]
        struct User {
            my_port: u16,
        }

        let user: User = cfg.try_raw_into().unwrap();

        assert_eq!(cfg.service.port, 8080);
        assert_eq!(cfg.service.hostname, "localhost");
        assert_eq!(cfg.service.debug, true);
        assert_eq!(
            cfg.service.grpc_server.timeout,
            Some(std::time::Duration::from_secs(2))
        );
        assert_eq!(user.my_port, 39999);
    }
}
