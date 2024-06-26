//! A library to generate "anisette" data. Docs are coming soon.
//!
//! If you want an async API, enable the `async` feature.
//!
//! If you want remote anisette, make sure the `remote-anisette` feature is enabled. (it's currently on by default)

use crate::adi_proxy::{ADIProxyAnisetteProvider, ConfigurableADIProxy};
use crate::anisette_headers_provider::AnisetteHeadersProvider;
use anyhow::Result;
use std::fmt::Formatter;
use std::path::PathBuf;

pub mod adi_proxy;
pub mod anisette_headers_provider;
pub mod store_services_core;

#[cfg(feature = "remote-anisette-v3")]
pub mod remote_anisette_v3;

#[cfg(target_os = "macos")]
pub mod aos_kit;

#[cfg(feature = "remote-anisette")]
pub mod remote_anisette;

#[allow(dead_code)]
pub struct AnisetteHeaders;

#[allow(dead_code)]
#[derive(Debug)]
enum AnisetteMetaError {
    #[allow(dead_code)]
    UnsupportedDevice,
    InvalidArgument(String),
}

impl std::fmt::Display for AnisetteMetaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnisetteMetaError::{self:?}")
    }
}

impl std::error::Error for AnisetteMetaError {}

pub const DEFAULT_ANISETTE_URL: &str = "https://ani.wesbryie.com/";

#[cfg(feature = "remote-anisette-v3")]
pub const DEFAULT_ANISETTE_URL_V3: &str = "https://ani.sidestore.io";

#[derive(Clone)]
pub struct AnisetteConfiguration {
    anisette_url: String,
    configuration_path: PathBuf,
}

impl Default for AnisetteConfiguration {
    fn default() -> Self {
        AnisetteConfiguration::new()
    }
}

impl AnisetteConfiguration {
    pub fn new() -> AnisetteConfiguration {
        AnisetteConfiguration {
            anisette_url: DEFAULT_ANISETTE_URL.to_string(),
            configuration_path: PathBuf::new(),
        }
    }

    pub fn anisette_url(&self) -> &String {
        &self.anisette_url
    }

    pub fn configuration_path(&self) -> &PathBuf {
        &self.configuration_path
    }

    pub fn set_anisette_url(mut self, anisette_url: String) -> AnisetteConfiguration {
        self.anisette_url = anisette_url;
        self
    }

    pub fn set_configuration_path(mut self, configuration_path: PathBuf) -> AnisetteConfiguration {
        self.configuration_path = configuration_path;
        self
    }
}

pub enum AnisetteHeadersProviderType {
    Local,
    Remote,
}

pub struct AnisetteHeadersProviderRes {
    pub provider: Box<dyn AnisetteHeadersProvider>,
    pub provider_type: AnisetteHeadersProviderType,
}

impl AnisetteHeadersProviderRes {
    pub fn local(provider: Box<dyn AnisetteHeadersProvider>) -> AnisetteHeadersProviderRes {
        AnisetteHeadersProviderRes {
            provider,
            provider_type: AnisetteHeadersProviderType::Local,
        }
    }

    pub fn remote(provider: Box<dyn AnisetteHeadersProvider>) -> AnisetteHeadersProviderRes {
        AnisetteHeadersProviderRes {
            provider,
            provider_type: AnisetteHeadersProviderType::Remote,
        }
    }
}

impl AnisetteHeaders {
    pub fn get_anisette_headers_provider(
        configuration: AnisetteConfiguration,
    ) -> Result<AnisetteHeadersProviderRes> {
        #[cfg(target_os = "macos")]
        if let Ok(prov) = aos_kit::AOSKitAnisetteProvider::new() {
            return Ok(AnisetteHeadersProviderRes::local(Box::new(prov)));
        }

        // TODO: handle Err because it will just go to remote anisette and not tell the user anything
        if let Ok(ssc_anisette_headers_provider) =
            AnisetteHeaders::get_ssc_anisette_headers_provider(configuration.clone())
        {
            return Ok(ssc_anisette_headers_provider);
        }

        #[cfg(feature = "remote-anisette")]
        return Ok(AnisetteHeadersProviderRes::remote(Box::new(
            remote_anisette::RemoteAnisetteProvider::new(configuration.anisette_url),
        )));

        #[cfg(not(feature = "remote-anisette"))]
        bail!(AnisetteMetaError::UnsupportedDevice)
    }

    pub fn get_ssc_anisette_headers_provider(
        configuration: AnisetteConfiguration,
    ) -> Result<AnisetteHeadersProviderRes> {
        let mut ssc_adi_proxy = store_services_core::StoreServicesCoreADIProxy::new(
            configuration.configuration_path(),
        )?;
        let config_path = configuration.configuration_path();
        ssc_adi_proxy.set_provisioning_path(config_path.to_str().ok_or(
            AnisetteMetaError::InvalidArgument("configuration.configuration_path".to_string()),
        )?)?;
        Ok(AnisetteHeadersProviderRes::local(Box::new(
            ADIProxyAnisetteProvider::new(ssc_adi_proxy, config_path.to_path_buf())?,
        )))
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use log::LevelFilter;
    use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

    pub fn init_logger() {
        if TermLogger::init(
            LevelFilter::Trace,
            ConfigBuilder::new()
                .set_target_level(LevelFilter::Error)
                .add_filter_allow_str("omnisette")
                .build(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )
        .is_ok()
        {}
    }

    #[cfg(not(feature = "async"))]
    #[test]
    fn fetch_anisette_auto() -> Result<()> {
        use crate::{AnisetteConfiguration, AnisetteHeaders};
        use log::info;
        use std::path::PathBuf;

        crate::tests::init_logger();

        let mut provider = AnisetteHeaders::get_anisette_headers_provider(
            AnisetteConfiguration::new()
                .set_configuration_path(PathBuf::new().join("anisette_test")),
        )?;
        info!(
            "Headers: {:?}",
            provider.provider.get_authentication_headers()?
        );
        Ok(())
    }
}
