/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2024.
 */

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

use async_channel::Receiver;
use cp_core::error::Error;
use cp_core::ok_or_return_error;
use tokio::signal;
use tokio::signal::unix::SignalKind;

use crate::error_kind::{FAILED_TO_DELETE_FILE, FAILED_TO_READ, FILE_NOT_FOUND, NOT_FOUND};
use crate::models::config_supply_request::ConfigSupplyRequest;
use crate::models::config_supply_response::ConfigSupplyResponse;
use crate::services::cleaner::clean_working_directory;
use crate::services::config_builder::ConfigBuilder;
use crate::services::downloader::Downloader;
use crate::services::packager::Packager;

pub struct ConfigSupplier {
    environments: Vec<String>,
    downloader: Arc<dyn Downloader + Send + Sync>,
    builder: Arc<dyn ConfigBuilder + Send + Sync>,
    packager: Arc<dyn Packager + Send + Sync>,
    working_path: PathBuf,
    last_stage: Option<String>,
}

impl ConfigSupplier {
    pub fn new(
        environments: Vec<String>,
        downloader: Arc<dyn Downloader + Send + Sync>,
        builder: Arc<dyn ConfigBuilder + Send + Sync>,
        packager: Arc<dyn Packager + Send + Sync>,
        working_path: PathBuf,
    ) -> Self {
        Self {
            environments,
            downloader,
            builder,
            packager,
            working_path,
            last_stage: None,
        }
    }

    //noinspection RsBorrowChecker
    // the no inspection is due to a false positive occurring at line 90 and 91.
    pub async fn run(mut self, receiver: Receiver<ConfigSupplyRequest>) {
        let mut sigint = match signal::unix::signal(SignalKind::interrupt()) {
            Ok(sigint) => sigint,
            Err(error) => {
                log::error!("failed to setup signal interrupt stream: {}", error);
                return;
            }
        };

        let mut sigterm = match signal::unix::signal(SignalKind::terminate()) {
            Ok(sigterm) => sigterm,
            Err(error) => {
                log::error!("failed to setup signal terminate stream: {}", error);
                return;
            }
        };

        let mut sigquit = match signal::unix::signal(SignalKind::quit()) {
            Ok(sigquit) => sigquit,
            Err(error) => {
                log::error!("failed to setup signal quit stream: {}", error);
                return;
            }
        };

        loop {
            tokio::select! {
                _ = sigint.recv() => {
                    return;
                },
                _ = sigterm.recv() => {
                    return;
                },
                _ = sigquit.recv() => {
                    return;
                },
                result = receiver.recv() => {
                    let request = match result {
                        Ok(request) => request,
                        Err(error) => {
                            log::warn!("failed to receive config supply request: {}", error);
                            return;
                        }
                    };

                    match request {
                        ConfigSupplyRequest::GetConfig {
                            stage,
                            environment,
                            component,
                            replier,
                        } => {
                            match self.initialize_stage(&stage) {
                                Ok(_) => (),
                                Err(error) => log::warn!("failed to initialize stage: {}", error),
                            }

                            let download_path: PathBuf = self.get_download_path();

                            let update_result = match self
                                .downloader
                                .is_new_version_available(&download_path, &stage)
                            {
                                Ok(is_new_version_available) => {
                                    if is_new_version_available {
                                        self.downloader.download(&download_path, &stage)
                                    } else {
                                        Ok(())
                                    }
                                }
                                Err(error) => Err(error),
                            };

                            if update_result.is_err() {
                                match replier.send(ConfigSupplyResponse::GetConfig { result: Err(update_result.unwrap_err()) }) {
                                    Ok(_) => (),
                                    Err(_) => log::warn!("failed to reply with the get configuration result"),
                                }

                                return;
                            }

                            let result = self.get_config(&environment, &component);

                            match replier.send(ConfigSupplyResponse::GetConfig { result }) {
                                Ok(_) => (),
                                Err(_) => log::warn!("failed to reply with the get configuration result"),
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_config(&self, environment: &str, component: &str) -> Result<Vec<u8>, Error> {
        let mut package_file_path = self.working_path.clone();
        package_file_path.push(environment);
        package_file_path.push(format!(
            "{}.{}",
            uuid::Uuid::new_v4(),
            self.packager.extension()
        ));

        let mut source_path = self.working_path.clone();
        source_path.push(environment);

        if !source_path.exists() {
            return Err(Error::new(
                NOT_FOUND,
                format!("environment '{}' does not exist", environment),
            ));
        }

        source_path.push(component);

        if !source_path.exists() {
            return Err(Error::new(
                NOT_FOUND,
                format!("component '{}' does not exist", component),
            ));
        }

        self.packager
            .package(&source_path, package_file_path.as_path())?;

        let mut package_file = ok_or_return_error!(
            File::open(&package_file_path),
            FILE_NOT_FOUND.to_string(),
            "failed to open package file: "
        );

        let mut buffer: Vec<u8> = Vec::new();

        ok_or_return_error!(
            package_file.read_to_end(&mut buffer),
            FAILED_TO_READ.to_string(),
            "failed to read package file: "
        );

        ok_or_return_error!(
            std::fs::remove_file(package_file_path),
            FAILED_TO_DELETE_FILE.to_string(),
            "failed to delete package file: "
        );

        Ok(buffer)
    }

    fn is_new_version_available(&self, stage: &str) -> Result<bool, Error> {
        let download_path = self.get_download_path();
        self.downloader
            .is_new_version_available(&download_path, stage)
    }

    fn get_download_path(&self) -> PathBuf {
        let mut download_path = self.working_path.clone();
        download_path.push("download");

        download_path
    }

    fn initialize_stage(&mut self, stage: &str) -> Result<(), Error> {
        match &self.last_stage {
            Some(last_stage) => {
                if last_stage != stage {
                    self.setup(stage)?;
                }
            }
            None => self.setup(stage)?,
        }

        Ok(())
    }

    fn setup(&mut self, stage: &str) -> Result<(), Error> {
        let download_path: PathBuf = self.get_download_path();
        let _ = std::fs::remove_dir_all(&self.working_path);
        match std::fs::create_dir_all(&self.working_path) {
            Ok(_) => (),
            Err(error) => {
                log::warn!(
                    "failed to create working path '{:?}': {}",
                    &self.working_path,
                    error
                );
            }
        }
        self.downloader.download(&download_path, stage)?;

        for environment in self.environments.as_slice() {
            let mut target_path = self.working_path.clone();
            target_path.push(environment);

            self.builder
                .build(environment, download_path.clone(), target_path)?;
        }

        self.last_stage = Some(stage.to_string());

        Ok(())
    }
}

impl Drop for ConfigSupplier {
    fn drop(&mut self) {
        match clean_working_directory(&self.working_path) {
            Ok(_) => (),
            Err(error) => log::warn!("failed to clean working directory: {}", error),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::Duration;

    use cp_core::test_base::get_unit_test_data_path;
    use tokio::time::timeout;

    use crate::error_kind::NOT_FOUND;
    use crate::models::config_supplier::ConfigSupplier;
    use crate::models::config_supply_request::ConfigSupplyRequest;
    use crate::models::config_supply_response::ConfigSupplyResponse;
    use crate::services::config_builder::{ConfigBuilder, MockConfigBuilder};
    use crate::services::downloader::{Downloader, MockDownloader};
    use crate::services::microconfig_config_builder::MicroconfigConfigBuilder;
    use crate::services::packager::{MockPackager, Packager};
    use crate::services::zip_packager::ZipPackager;
    use crate::test_base::get_git_downloader;

    pub const TEST_STAGE: &str = "dummy";
    pub const ALT_TEST_STAGE: &str = "dummy-2";

    #[test]
    pub fn setup_builds_all_environments() {
        let working_dir = format!("./{}", uuid::Uuid::new_v4());
        let mut supplier = get_config_supplier(working_dir.clone());

        supplier
            .setup(TEST_STAGE)
            .expect("failed to setup config supplier");

        for environment in get_environments() {
            assert!(std::fs::metadata(format!("{}/{}", working_dir, environment)).is_ok());
            assert!(std::fs::metadata(format!(
                "{}/{}/dummy/application.yaml",
                working_dir, environment
            ))
            .is_ok());
        }
    }

    #[test]
    pub fn get_config_returns_bytes_of_zip_file() {
        let working_dir = format!("./{}", uuid::Uuid::new_v4());
        let mut supplier = get_config_supplier(working_dir);
        supplier
            .setup(TEST_STAGE)
            .expect("failed to setup config manager");

        let result = supplier.get_config("dummy", "dummy");

        match result {
            Ok(data) => assert!(!data.is_empty()),
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[tokio::test]
    pub async fn run_get_config_request_sends_config() {
        let expected_file_bytes: Vec<u8> = vec![];
        let (sender, replier, reply_receiver) = prepare_config_supplier();
        sender
            .send(ConfigSupplyRequest::GetConfig {
                stage: TEST_STAGE.to_string(),
                environment: "dummy".to_string(),
                component: "dummy".to_string(),
                replier,
            })
            .await
            .unwrap();

        match timeout(Duration::from_secs(1u64), reply_receiver)
            .await
            .unwrap()
        {
            Ok(result) => match result {
                ConfigSupplyResponse::GetConfig { result } => {
                    let bytes: Vec<u8> =
                        result.expect("failed to get expected file bytes from config result");

                    assert_eq!(expected_file_bytes, bytes);
                }
                _ => panic!("unexpected result received"),
            },
            Err(error) => panic!("failed to receive get config result: {}", error),
        }
    }

    #[tokio::test]
    pub async fn get_config_returns_error_if_component_does_not_exist() {
        let (sender, replier, reply_receiver) = prepare_config_supplier();
        sender
            .send(ConfigSupplyRequest::GetConfig {
                stage: TEST_STAGE.to_string(),
                environment: "dummy".to_string(),
                component: "non-existent".to_string(),
                replier,
            })
            .await
            .unwrap();

        match timeout(Duration::from_secs(1u64), reply_receiver)
            .await
            .expect("timed out getting config")
            .expect("expected response but got a 'RecvError'")
        {
            ConfigSupplyResponse::GetConfig { result } => {
                let error = result.expect_err("expected error got ok getting configuration");

                assert_eq!(NOT_FOUND, error.error_kind());
                assert!(error.message().contains("component"))
            }
            _ => panic!("got an unexpected response for 'GetConfig'"),
        }
    }

    #[tokio::test]
    pub async fn get_config_returns_error_if_environment_does_not_exist() {
        let (sender, replier, reply_receiver) = prepare_config_supplier();
        sender
            .send(ConfigSupplyRequest::GetConfig {
                stage: TEST_STAGE.to_string(),
                environment: "non-existent".to_string(),
                component: "dummy".to_string(),
                replier,
            })
            .await
            .unwrap();

        match timeout(Duration::from_secs(1u64), reply_receiver)
            .await
            .expect("timed out getting config")
            .expect("expected response but got a 'RecvError'")
        {
            ConfigSupplyResponse::GetConfig { result } => {
                let error = result.expect_err("expected error got ok getting configuration");

                assert_eq!(NOT_FOUND, error.error_kind());
                assert!(error.message().contains("environment"));
            }
            _ => panic!("got an unexpected response for 'GetConfig'"),
        }
    }

    pub fn mock_dependencies() -> (
        Arc<dyn Downloader + Send + Sync>,
        Arc<dyn ConfigBuilder + Send + Sync>,
        Arc<dyn Packager + Send + Sync>,
    ) {
        let mut mock_downloader = MockDownloader::new();
        mock_downloader
            .expect_is_new_version_available()
            .returning(|_, _| Ok(false));
        mock_downloader.expect_download().returning(|_, _| Ok(()));
        let mut mock_builder = MockConfigBuilder::new();
        mock_builder.expect_build().returning(|_, _, target_path| {
            let mut dummy_component_path = target_path.clone();
            dummy_component_path.push("dummy");
            dummy_component_path.push("components");
            dummy_component_path.push("dummy");
            std::fs::create_dir_all(dummy_component_path)?;

            Ok(())
        });
        let mut mock_packager = MockPackager::new();
        mock_packager
            .expect_package()
            .returning(|source_path, target_file| {
                std::fs::create_dir_all(source_path).expect("failed to create source directory");
                std::fs::File::create(target_file).expect("failed to create target file");

                Ok(())
            });
        mock_packager
            .expect_extension()
            .returning(|| "zip".to_string());
        let downloader: Arc<dyn Downloader + Send + Sync> = Arc::new(mock_downloader);
        let builder: Arc<dyn ConfigBuilder + Send + Sync> = Arc::new(mock_builder);
        let packager: Arc<dyn Packager + Send + Sync> = Arc::new(mock_packager);

        (downloader, builder, packager)
    }

    pub fn get_environments() -> Vec<String> {
        vec![
            "dummy".to_string(),
            "development".to_string(),
            "staging".to_string(),
            "production".to_string(),
        ]
    }

    fn get_config_supplier(working_dir: String) -> ConfigSupplier {
        let downloader: Arc<dyn Downloader + Send + Sync> =
            Arc::new(get_git_downloader(get_unit_test_data_path(file!())));
        let builder: Arc<dyn ConfigBuilder + Send + Sync> =
            Arc::new(MicroconfigConfigBuilder::default());
        let working_path: PathBuf = working_dir.into();
        let packager: Arc<dyn Packager + Send + Sync> = Arc::new(ZipPackager::default());

        ConfigSupplier::new(
            get_environments(),
            downloader,
            builder,
            packager,
            working_path,
        )
    }

    fn prepare_config_supplier() -> (
        async_channel::Sender<ConfigSupplyRequest>,
        tokio::sync::oneshot::Sender<ConfigSupplyResponse>,
        tokio::sync::oneshot::Receiver<ConfigSupplyResponse>,
    ) {
        let working_dir = uuid::Uuid::new_v4().to_string();
        let (downloader, builder, packager) = mock_dependencies();
        let working_path: PathBuf = working_dir.into();
        let config_supplier = ConfigSupplier::new(
            get_environments(),
            downloader,
            builder,
            packager,
            working_path,
        );
        let (sender, receiver) = async_channel::bounded::<ConfigSupplyRequest>(1024usize);
        tokio::spawn(async move {
            config_supplier.run(receiver).await;
        });
        let (replier, reply_receiver) = tokio::sync::oneshot::channel::<ConfigSupplyResponse>();

        (sender, replier, reply_receiver)
    }
}
