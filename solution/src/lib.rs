use tari_template_lib::prelude::*;

const FEE: Amount = Amount(10);

#[template]
mod template {
    use super::*;

    pub struct Monerokon {
        counter: u32,
        // 🏋️ EXERCISE 3b: Add two vault called 'supply_vault' and 'fee_vault'
        supply_vault: Vault,
        fee_vault: Vault,
        // 🏋️ EXERCISE 5b: Create a Non-Fungible resource with two NFTs in a new vault named 'nft_vault'
        nft_vault: Vault,
        // 🏋️ EXERCISE 6b: Add a confidential vault called 'confidential_vault'
        confidential_vault: Vault,
    }

    impl Monerokon {
        /// Construct the component with an initial supply of fungible tokens.
        pub fn new(
            initial_supply: Amount,
            confidential_initial_supply: ConfidentialOutputStatement,
        ) -> Component<Self> {
            // 🏋️ EXERCISE 3a: Create a fungible resource with an initial supply
            let bucket = ResourceBuilder::fungible()
                .with_token_symbol("MNROKN")
                .add_metadata("name", "Monerokon Coin")
                .initial_supply(initial_supply);

            // 🏋️ EXERCISE 5: Create a Non-Fungible resource with two NFTs in a new vault named 'nft_vault'
            let nfts = ResourceBuilder::non_fungible()
                .add_metadata("name", "Monerokon NFT")
                .initial_supply([NonFungibleId::Uint64(1), NonFungibleId::Uint64(2)]);

            // 🏋️ EXERCISE 6a: Create a confidential resource with an initial supply and add a vault to the component called 'confidential_vault'
            let confidential_bucket = ResourceBuilder::confidential()
                .initial_supply(confidential_initial_supply);

            // 🏋️ EXERCISE 3c: Deposit the initial tokens into a supply vault and create an empty fee vault
            Component::new(Self {
                fee_vault: Vault::new_empty(XTR2),
                supply_vault: Vault::from_bucket(bucket),
                counter: 0,
                nft_vault: Vault::from_bucket(nfts),
                confidential_vault: Vault::from_bucket(confidential_bucket),
            })
            // 🏋️ EXERCISE 4b: allow anyone to call the "withdraw" method
            .with_access_rules(
                ComponentAccessRules::new()
                    .add_method_rule("withdraw", AccessRule::AllowAll)
                    .add_method_rule("withdraw_confidential", AccessRule::AllowAll),
            )
            .create()
        }

        pub fn get_balance(&self) -> Amount {
            // 🏋️ EXERCISE 3c: Return the supply vault balance
            self.supply_vault.balance()
        }

        pub fn withdraw(&mut self, fee: Bucket, amount: Amount) -> Bucket {
            // 🏋️ EXERCISE 4a: check fee amount and deposit then in the fee_vault. Withdraw requested amount from supply vault and return the Bucket.
            //                Increment the counter.
            assert!(fee.amount() >= FEE, "fee is too low");
            self.fee_vault.deposit(fee);
            self.counter += 1;
            self.supply_vault.withdraw(amount)
        }

        pub fn withdraw_confidential(
            &mut self,
            fee: Bucket,
            withdraw_proof: ConfidentialWithdrawProof,
        ) -> Bucket {
            // 🏋️ EXERCISE 6b: check fee amount and deposit then in the fee_vault. Withdraw requested amount from confidential vault and return the Bucket.
            //                Increment the counter.
            fee.assert_contains_no_confidential_funds();
            assert!(fee.amount() >= FEE, "fee is too low");
            self.fee_vault.deposit(fee);
            self.counter += 1;
            self.confidential_vault
                .withdraw_confidential(withdraw_proof)
        }

        // 🏋️ EXERCISE 7a: Mint fungible tokens and deposit them in the supply_vault
        pub fn mint_fungible(&self, fungible_amount: Amount) {
            let manager = ResourceManager::get(self.supply_vault.resource_address());
            let bucket = manager.mint_fungible(fungible_amount);
            self.supply_vault.deposit(bucket);
        }

        // 🏋️ EXERCISE 7b: Mint fungible tokens and deposit them in the nft_vault
        pub fn mint_non_fungible(&self, nft: NonFungibleId) {
            #[derive(serde::Serialize)]
            struct MyData {
                data: String,
            }

            let manager = ResourceManager::get(self.nft_vault.resource_address());
            let bucket = manager.mint_non_fungible(
                nft,
                &(),
                &MyData {
                    data: "beep-boop".to_string(),
                },
            );
            self.nft_vault.deposit(bucket);
        }

        // 🏋️ EXERCISE 7c: Mint confidential tokens and deposit them in the confidential_vault
        pub fn mint_confidential(&self, confidential: ConfidentialOutputStatement) {
            let manager = ResourceManager::get(self.confidential_vault.resource_address());
            let bucket = manager.mint_confidential(confidential);
            self.confidential_vault.deposit(bucket);
        }

        // 🏋️ EXERCISE 2a: Implement method to return the counter value
        pub fn counter(&self) -> u32 {
            self.counter
        }

        // 🏋️ EXERCISE 2b: Implement method to increase the counter value by 1
        pub fn increase(&mut self) {
            self.counter += 1;
        }
    }
}
