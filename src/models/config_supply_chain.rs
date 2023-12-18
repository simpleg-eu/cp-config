/*
 * Copyright (c) Gabriel Amihalachioaie, SimpleG 2023.
 */

use async_channel::{Receiver, Sender};

use cp_core::error::Error;

use crate::models::config_supplier::ConfigSupplier;
use crate::models::config_supplier_init::ConfigSupplierInit;
use crate::models::config_supply_request::ConfigSupplyRequest;

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
}
