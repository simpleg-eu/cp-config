/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use async_channel::{Receiver, Sender};
use cp_core::error::Error;

use crate::error::ConfigError;
use crate::error_kind::CHANNEL_COMMUNICATION_FAILURE;
use crate::models::config_supplier::ConfigSupplier;
use crate::models::config_supplier_init::ConfigSupplierInit;
use crate::models::config_supply_request::ConfigSupplyRequest;
use crate::models::config_supply_response::ConfigSupplyResponse;
use crate::return_error;

pub struct ConfigSupplyChain {
    sender: Sender<ConfigSupplyRequest>,
    receiver: Receiver<ConfigSupplyRequest>,
    config_supplier_init: ConfigSupplierInit,
}

impl ConfigSupplyChain {
    pub fn try_new(
        suppliers_count: usize,
        config_supplier_init: ConfigSupplierInit,
    ) -> Result<Self, Error> {
        let (sender, receiver) = async_channel::bounded::<ConfigSupplyRequest>(1024usize);

        let supply_chain = Self {
            sender,
            receiver,
            config_supplier_init,
        };

        for _ in 0usize..suppliers_count {
            supply_chain.add_supplier()?;
        }

        Ok(supply_chain)
    }

    pub async fn get_config(&self, environment: &str, component: &str) -> Result<Vec<u8>, Error> {
        let (replier, reply_receiver) = tokio::sync::oneshot::channel::<ConfigSupplyResponse>();

        return_error!(
            self.sender
                .send(ConfigSupplyRequest::GetConfig {
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
                CHANNEL_COMMUNICATION_FAILURE.to_string(),
                format!(
                    "received an unexpected response type for get config request: {:?}",
                    response
                ),
            )),
        }
    }

    fn add_supplier(&self) -> Result<(), Error> {
        let working_path = uuid::Uuid::new_v4().to_string();
        let supplier = ConfigSupplier::try_new(
            self.config_supplier_init.environments.clone(),
            self.config_supplier_init.downloader.clone(),
            self.config_supplier_init.builder.clone(),
            self.config_supplier_init.packager.clone(),
            working_path.into(),
            self.config_supplier_init.stage.clone(),
        )?;

        let receiver_clone = self.receiver.clone();
        tokio::spawn(async move {
            supplier.run(receiver_clone).await;
        });

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::models::config_supplier::tests::get_environments;
    use crate::models::config_supplier::tests::mock_dependencies;
    use crate::models::config_supplier_init::ConfigSupplierInit;
    use crate::models::config_supply_chain::ConfigSupplyChain;

    #[tokio::test]
    pub async fn try_new_creates_specified_config_suppliers() {
        let expected_suppliers: usize = 2usize;
        let (downloader, builder, packager) = mock_dependencies();

        let supply_chain = ConfigSupplyChain::try_new(
            expected_suppliers,
            ConfigSupplierInit {
                environments: get_environments(),
                downloader,
                builder,
                packager,
                stage: "dummy".to_string(),
            },
        )
        .unwrap();

        // + 1 in order to include the receiver held within the ConfigSupplyChain struct.
        assert_eq!(expected_suppliers + 1, supply_chain.sender.receiver_count());
    }

    #[tokio::test]
    pub async fn get_config_replies_with_expected_bytes() {
        let expected_bytes: Vec<u8> = vec![];
        let suppliers_count: usize = 2usize;
        let (downloader, builder, packager) = mock_dependencies();
        let supply_chain = ConfigSupplyChain::try_new(
            suppliers_count,
            ConfigSupplierInit {
                environments: get_environments(),
                downloader,
                builder,
                packager,
                stage: "dummy".to_string(),
            },
        )
        .unwrap();

        let config = supply_chain.get_config("dummy", "dummy").await.unwrap();

        assert_eq!(expected_bytes, config);
    }
}
