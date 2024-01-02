/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2024.
 */

use std::collections::HashMap;

use async_channel::{Receiver, Sender};
use cp_core::error::Error;

use crate::error::ConfigError;
use crate::error_kind::UNEXPECTED_RESPONSE_TYPE;
use crate::models::config_supplier::ConfigSupplier;
use crate::models::config_supplier_init::ConfigSupplierInit;
use crate::models::config_supply_request::ConfigSupplyRequest;
use crate::models::config_supply_response::ConfigSupplyResponse;
use crate::return_error;

pub struct ConfigSupplyChain {
    /// Contains stages as keys in order to have the most accessed stages as static avoiding repetitive write operations.
    static_suppliers: HashMap<String, (Sender<ConfigSupplyRequest>, Receiver<ConfigSupplyRequest>)>,
    dynamic_supplier_sender: Sender<ConfigSupplyRequest>,
    dynamic_supplier_receiver: Receiver<ConfigSupplyRequest>,
    config_supplier_init: ConfigSupplierInit,
    suppliers_count: usize,
}

impl ConfigSupplyChain {
    pub fn try_new(
        suppliers_count: usize,
        static_stages: Vec<String>,
        config_supplier_init: ConfigSupplierInit,
    ) -> Result<Self, Error> {
        let mut static_suppliers = HashMap::new();

        for static_stage in static_stages {
            let (static_sender, static_receiver) =
                async_channel::bounded::<ConfigSupplyRequest>(1024usize);
            static_suppliers.insert(static_stage, (static_sender, static_receiver));
        }

        let (sender, receiver) = async_channel::bounded::<ConfigSupplyRequest>(1024usize);

        let supply_chain = Self {
            static_suppliers,
            dynamic_supplier_sender: sender,
            dynamic_supplier_receiver: receiver.clone(),
            config_supplier_init,
            suppliers_count,
        };

        for _ in 0usize..suppliers_count {
            supply_chain.add_supplier(receiver.clone())?;
        }

        Ok(supply_chain)
    }

    pub async fn get_config(
        &self,
        stage: &str,
        environment: &str,
        component: &str,
    ) -> Result<Vec<u8>, Error> {
        match self.static_suppliers.get(stage) {
            Some((static_sender, static_receiver)) => {
                self.get_config_for_sender_and_receiver(
                    stage,
                    environment,
                    component,
                    static_sender,
                    static_receiver,
                )
                .await
            }
            None => {
                self.get_config_for_sender_and_receiver(
                    stage,
                    environment,
                    component,
                    &self.dynamic_supplier_sender,
                    &self.dynamic_supplier_receiver,
                )
                .await
            }
        }
    }

    async fn get_config_for_sender_and_receiver(
        &self,
        stage: &str,
        environment: &str,
        component: &str,
        sender: &Sender<ConfigSupplyRequest>,
        receiver: &Receiver<ConfigSupplyRequest>,
    ) -> Result<Vec<u8>, Error> {
        let current_suppliers_count = sender.receiver_count() - 1;
        if current_suppliers_count < self.suppliers_count {
            for _ in 0..(self.suppliers_count - current_suppliers_count) {
                self.add_supplier(receiver.clone())?;
            }
        }

        let (replier, reply_receiver) = tokio::sync::oneshot::channel::<ConfigSupplyResponse>();

        return_error!(
            sender
                .send(ConfigSupplyRequest::GetConfig {
                    stage: stage.to_string(),
                    environment: environment.to_string(),
                    component: component.to_string(),
                    replier,
                })
                .await
        );

        let response = return_error!(reply_receiver.await);

        match response {
            ConfigSupplyResponse::GetConfig { result } => result,
            _ => Err(Error::new(
                UNEXPECTED_RESPONSE_TYPE.to_string(),
                format!(
                    "received an unexpected response type for get config request: {:?}",
                    response
                ),
            )),
        }
    }

    fn add_supplier(&self, receiver: Receiver<ConfigSupplyRequest>) -> Result<(), Error> {
        let working_path = uuid::Uuid::new_v4().to_string();
        let supplier = ConfigSupplier::new(
            self.config_supplier_init.environments.clone(),
            self.config_supplier_init.downloader.clone(),
            self.config_supplier_init.builder.clone(),
            self.config_supplier_init.packager.clone(),
            working_path.into(),
        );

        tokio::spawn(async move {
            supplier.run(receiver).await;
        });

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::models::config_supplier::tests::{get_environments, mock_dependencies, TEST_STAGE};
    use crate::models::config_supplier_init::ConfigSupplierInit;
    use crate::models::config_supply_chain::ConfigSupplyChain;

    #[tokio::test]
    pub async fn try_new_creates_specified_config_suppliers() {
        let expected_suppliers: usize = 2usize;
        let (downloader, builder, packager) = mock_dependencies();

        let supply_chain = ConfigSupplyChain::try_new(
            expected_suppliers,
            vec!["main".to_string()],
            ConfigSupplierInit {
                environments: get_environments(),
                downloader,
                builder,
                packager,
            },
        )
        .unwrap();

        // + 1 in order to include the receiver held within the ConfigSupplyChain struct.
        assert_eq!(
            expected_suppliers + 1,
            supply_chain.dynamic_supplier_sender.receiver_count()
        );
    }

    #[tokio::test]
    pub async fn get_config_replies_with_expected_bytes() {
        let expected_bytes: Vec<u8> = vec![];
        let suppliers_count: usize = 2usize;
        let (downloader, builder, packager) = mock_dependencies();
        let supply_chain = ConfigSupplyChain::try_new(
            suppliers_count,
            vec!["main".to_string()],
            ConfigSupplierInit {
                environments: get_environments(),
                downloader,
                builder,
                packager,
            },
        )
        .unwrap();

        let config = supply_chain
            .get_config(TEST_STAGE, "dummy", "dummy")
            .await
            .unwrap();

        assert_eq!(expected_bytes, config);
    }

    #[tokio::test]
    pub async fn get_config_initializes_new_suppliers() {
        let suppliers_count: usize = 2usize;
        let expected_suppliers: usize = 4usize;
        let (downloader, builder, packager) = mock_dependencies();
        let mut supply_chain = ConfigSupplyChain::try_new(
            suppliers_count,
            vec!["main".to_string()],
            ConfigSupplierInit {
                environments: get_environments(),
                downloader,
                builder,
                packager,
            },
        )
        .unwrap();
        supply_chain.suppliers_count = expected_suppliers;

        let _ = supply_chain
            .get_config(TEST_STAGE, "dummy", "dummy")
            .await
            .unwrap();

        // + 1 in order to include the receiver held within the ConfigSupplyChain struct.
        assert_eq!(
            expected_suppliers + 1,
            supply_chain.dynamic_supplier_sender.receiver_count()
        );
    }
}
