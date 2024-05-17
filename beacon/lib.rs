#![cfg_attr(not(feature = "std"), no_std, no_main)]
pub use self::gateway::{Gateway, GatewayRef};

#[ink::contract]
mod gateway {

    use ink::storage::Mapping;
    use ink::prelude::vec::Vec;

    type OpaqueSignature = Vec<u8>;

    #[ink(storage)]
    pub struct Gateway {
        /// a trusted origin
        authorized_caller: AccountId,
        /// stores a subset of ETF signatures/proofs
        blocks: Mapping<BlockNumber, OpaqueSignature>,
        /// latest block number encountered
        latest_block_number: BlockNumber,
    }

    #[derive(Clone, PartialEq, Debug, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum Error {
        InvalidOrigin,
    }

    impl Gateway {
        #[ink(constructor)]
        pub fn new(authorized_caller: AccountId) -> Self {
            Self {
                authorized_caller,
                blocks: Mapping::default(),
                latest_block_number: 0,
            }
        }

        #[ink(message)]
        pub fn get_latest_block_number(&self) -> BlockNumber {
            self.latest_block_number
        }

        #[ink(message, payable)]
        pub fn write_block(
            &mut self, 
            best_etf_block_number: BlockNumber,
            serialized_sig: Vec<u8>,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            if caller != self.authorized_caller {
                return Err(Error::InvalidOrigin);
            }

            self.blocks.insert(best_etf_block_number, &serialized_sig);
            self.latest_block_number = best_etf_block_number;

            Ok(())
        }

        #[ink(message)]
        pub fn read_block(&self, block_number: BlockNumber) -> Option<Vec<u8>> {
            self.blocks.get(block_number)
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        // /// We test if the default constructor does its job.
        // #[ink::test]
        // fn default_works() {
        //     let gateway = Gateway::default();
        //     assert_eq!(gateway.get(), false);
        // }

        // /// We test a simple use case of our contract.
        // #[ink::test]
        // fn it_works() {
        //     let mut gateway = Gateway::new(false);
        //     assert_eq!(gateway.get(), false);
        //     gateway.flip();
        //     assert_eq!(gateway.get(), true);
        // }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        // /// We test that we can upload and instantiate the contract using its default constructor.
        // #[ink_e2e::test]
        // async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        //     // Given
        //     let mut constructor = GatewayRef::default();

        //     // When
        //     let contract = client
        //         .instantiate("gateway", &ink_e2e::alice(), &mut constructor)
        //         .submit()
        //         .await
        //         .expect("instantiate failed");
        //     let call_builder = contract.call_builder::<Gateway>();

        //     // Then
        //     let get = call_builder.get();
        //     let get_result = client.call(&ink_e2e::alice(), &get).dry_run().await?;
        //     assert!(matches!(get_result.return_value(), false));

        //     Ok(())
        // }

        // /// We test that we can read and write a value from the on-chain contract.
        // #[ink_e2e::test]
        // async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        //     // Given
        //     let mut constructor = GatewayRef::new(false);
        //     let contract = client
        //         .instantiate("gateway", &ink_e2e::bob(), &mut constructor)
        //         .submit()
        //         .await
        //         .expect("instantiate failed");
        //     let mut call_builder = contract.call_builder::<Gateway>();

        //     let get = call_builder.get();
        //     let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
        //     assert!(matches!(get_result.return_value(), false));

        //     // When
        //     let flip = call_builder.flip();
        //     let _flip_result = client
        //         .call(&ink_e2e::bob(), &flip)
        //         .submit()
        //         .await
        //         .expect("flip failed");

        //     // Then 
        //     let get = call_builder.get();
        //     let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
        //     assert!(matches!(get_result.return_value(), true));

        //     Ok(())
        // }
    }
}
