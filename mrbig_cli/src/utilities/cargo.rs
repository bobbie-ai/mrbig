// ############################################################################
// #                                                                          #
// # mrbig_cli/src/utilities/cargo.rs                                         #
// #                                                                          #
// # Handcrafted with love by MrBig Mobsters                                  #
// # All rights reserved                                                      #
// #                                                                          #
// #                                                                          #
// # Description: Routines for manipulating (meta-)data of Cargo crates.      #
// # Credits:     Much inspired from wasm-pack tool.                          #
// ############################################################################

//! Cargo crate manipulation module.
//!
//! This modules provides some useful features to read Cargo crate meta-data.

#![allow(
    clippy::new_ret_no_self,
    clippy::needless_pass_by_value,
    clippy::redundant_closure,
    dead_code,
    unused_imports
)]

// Import modules
use anyhow::Result;
use cargo_metadata::Metadata;
use chrono::offset;
use chrono::DateTime;
use curl::easy;
use serde::{self, Deserialize};
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use strsim::levenshtein;

// Local constants definition
const APP_METADATA_KEY: &str = "package.metadata.mrbig";
const APP_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const APP_REPO_URL: &str = "https://github.com/bobbie-ai/mrbig";
const APP_CRATE_URL: &str = "https://crates.io/api/v1/crates/mrbig";

/// Cargo crate meta-data store
///
/// This structure stores the information of a crate.
pub struct CrateMetadata {
    data: Metadata,
    current_idx: usize,
    manifest: CargoManifest,
    out_name: Option<String>,
}

#[doc(hidden)]
#[derive(Deserialize)]
pub struct CargoManifest {
    package: CargoPackage,
}

#[derive(Deserialize)]
struct CargoPackage {
    name: String,
    version: String,
    description: Option<String>,
    license: Option<String>,
    repository: Option<String>,
    homepage: Option<String>,
}

/// Structure for storing information received from crates.io
#[derive(Deserialize, Debug)]
pub struct Crate {
    #[serde(rename = "crate")]
    latest_version: String,
}

impl Crate {
    /// Returns the latest application version
    pub fn get_latest_version() -> Result<Option<String>> {
        let current_time = chrono::offset::Local::now();
        let old_metadata_file = Self::return_stamp_file();

        match old_metadata_file {
            Some(ref file_contents) => {
                let last_updated = Self::return_stamp_file_value(&file_contents, "created")
                    .and_then(|t| DateTime::parse_from_str(t.as_str(), "%+").ok());

                last_updated
                    .map(|last_updated| {
                        if current_time.signed_duration_since(last_updated).num_hours() > 24 {
                            Self::return_api_call_result(current_time).map(Some)
                        } else {
                            Ok(Self::return_stamp_file_value(&file_contents, "version"))
                        }
                    })
                    .unwrap_or_else(|| Ok(None))
            }

            None => Self::return_api_call_result(current_time).map(Some),
        }
    }

    // Handles calls back from the Crates.io API server when asking for the latest crate version
    fn return_api_call_result(current_time: DateTime<offset::Local>) -> Result<String> {
        let version = Self::return_latest_version();

        // We always override the stamp file with the current time because we don't
        // want to hit the API all the time if it fails. It should follow the same
        // "policy" as the success. This means that the 24 hours rate limiting
        // will be active regardless if the check succeeded or failed.
        match version {
            Ok(ref version) => Self::override_stamp_file(current_time, Some(&version)).ok(),
            Err(_) => Self::override_stamp_file(current_time, None).ok(),
        };

        version
    }

    /// Returns wasm-pack latest version (if it's received) by executing check_wasm_pack_latest_version function.
    fn return_latest_version() -> Result<String> {
        Self::check_latest_version_in_creates_io().map(|crate_info| crate_info.latest_version)
    }

    /// Return stamp file where metadata is stored.
    fn return_stamp_file() -> Option<String> {
        if let Ok(path) = env::current_exe() {
            if let Ok(file) = fs::read_to_string(path.with_extension("stamp")) {
                return Some(file);
            }
        }
        None
    }

    /// Read the stamp file and return value assigned to a certain key.
    fn return_stamp_file_value(file: &str, word: &str) -> Option<String> {
        let created = file
            .lines()
            .find(|line| line.starts_with(word))
            .and_then(|l| l.split_whitespace().nth(1));

        created.map(|s| s.to_string())
    }

    // Override the creation time of the stamp file used to store meta-data
    fn override_stamp_file(
        current_time: DateTime<offset::Local>,
        version: Option<&str>,
    ) -> Result<()> {
        let path = env::current_exe()?;

        // open a stamp file in which to store transient data
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open(path.with_extension("stamp"))?;

        file.set_len(0)?;

        write!(file, "created {:?}", current_time)?;

        if let Some(version) = version {
            write!(file, "\nversion {}", version)?;
        }

        Ok(())
    }

    /// Invokes the `crates.io` repository's API for checking for the latest version of the application
    fn check_latest_version_in_creates_io() -> Result<Crate> {
        let url = APP_CRATE_URL; //"https://crates.io/api/v1/crates/mrbig";

        let mut easy = easy::Easy2::new(CurlCollector(Vec::new()));

        easy.useragent(&format!(
            "mrbig/{} ({})",
            APP_VERSION.unwrap_or_else(|| "unknown"),
            APP_REPO_URL
        ))?;

        easy.url(url)?;
        easy.get(true)?;
        easy.perform()?;

        let status_code = easy.response_code()?;

        if 200 <= status_code && status_code < 300 {
            let contents = easy.get_ref();
            let result = String::from_utf8_lossy(&contents.0);

            Ok(serde_json::from_str(result.into_owned().as_str())?)
        } else {
            Err(anyhow!("Received a bad HTTP status code ({}) when checking for newer `Mr. Big` version at: {}", status_code,url))
        }
    }
} // end of `Create` structure implementation

// Vector for storing results from Curl invocations
struct CurlCollector(Vec<u8>);

impl easy::Handler for CurlCollector {
    fn write(&mut self, data: &[u8]) -> Result<usize, easy::WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}
