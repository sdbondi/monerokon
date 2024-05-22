use tari_template_lib::prelude::*;
use tari_template_test_tooling::support::confidential::{
    generate_confidential_proof, generate_withdraw_proof_with_inputs, WithdrawProofOutput,
};
use tari_template_test_tooling::TemplateTest;
use tari_transaction::Transaction;

const INITIAL_SUPPLY: Amount = Amount(1_000_000);

#[test]
fn exercise_1_initial_states() {
    let mut test = TemplateTest::new(["."]);

    let template = test.get_module("Monerokon");
    let num_args = template
        .template_def()
        .functions()
        .iter()
        .find(|f| f.name == "new")
        .unwrap()
        .arguments
        .len();

    // Construct the component
    let component_address: ComponentAddress = if num_args == 0 {
        test.call_function("Monerokon", "new", args![], vec![])
    } else {
        let (output, _, _) = generate_confidential_proof(INITIAL_SUPPLY, None);
        test.call_function("Monerokon", "new", args![INITIAL_SUPPLY, output], vec![])
    };

    let counter: u32 = test.extract_component_value(component_address, "$.counter");
    assert_eq!(counter, 0, "Counter was not initialize to 0");
}

#[test]
fn exercise_2_counter() {
    let mut test = TemplateTest::new(["."]);

    let template = test.get_module("Monerokon");
    let num_args = template
        .template_def()
        .functions()
        .iter()
        .find(|f| f.name == "new")
        .unwrap()
        .arguments
        .len();

    // Construct the component
    let component_address: ComponentAddress = if num_args == 0 {
        test.call_function("Monerokon", "new", args![], vec![])
    } else {
        let (output, _, _) = generate_confidential_proof(INITIAL_SUPPLY, None);
        test.call_function("Monerokon", "new", args![INITIAL_SUPPLY, output], vec![])
    };

    // Get proof of ownership for the component.
    let proof = test.get_test_proof();

    // Call `value` method on component
    let counter: u32 = test.call_method(component_address, "counter", args![], vec![proof.clone()]);
    assert_eq!(counter, 0);

    // Increase the counter (Mutate component state)
    test.call_method::<()>(component_address, "increase", args![], vec![proof.clone()]);
    test.call_method::<()>(component_address, "increase", args![], vec![proof.clone()]);

    // Assert the counter was increased
    let counter: u32 = test.call_method(component_address, "counter", args![], vec![proof]);
    assert_eq!(counter, 2);
}

#[test]
fn exercise_3_fungible_resource() {
    let mut test = TemplateTest::new(["."]);

    // Construct the component
    let (output, _, _) = generate_confidential_proof(INITIAL_SUPPLY, None);
    // Construct the component
    let component_address: ComponentAddress =
        test.call_function("Monerokon", "new", args![INITIAL_SUPPLY, output], vec![]);

    let vault_id: VaultId = test.extract_component_value(component_address, "$.supply_vault");
    let vault = test.read_only_state_store().get_vault(&vault_id).unwrap();
    assert_eq!(vault.resource_type(), ResourceType::Fungible);
    assert_ne!(*vault.resource_address(), XTR2);

    // Get proof of ownership for the component.
    let proof = test.get_test_proof();

    // Call `get_balance` method on component
    let value: Amount = test.call_method(
        component_address,
        "get_balance",
        args![],
        vec![proof.clone()],
    );
    assert_eq!(value, INITIAL_SUPPLY);

    let vault_id: VaultId = test.extract_component_value(component_address, "$.fee_vault");
    let vault = test.read_only_state_store().get_vault(&vault_id).unwrap();
    assert_eq!(*vault.resource_address(), XTR2);
}

#[test]
fn exercise_4_withdraw_and_fees() {
    let mut test = TemplateTest::new(["."]);

    // Construct the component
    let (output, _, _) = generate_confidential_proof(INITIAL_SUPPLY, None);
    let component_address: ComponentAddress =
        test.call_function("Monerokon", "new", args![INITIAL_SUPPLY, output], vec![]);

    let (account, account_proof, account_secret) = test.create_funded_account();

    let vault_id: VaultId = test.extract_component_value(component_address, "$.supply_vault");
    let vault = test.read_only_state_store().get_vault(&vault_id).unwrap();
    let fungible_resource = *vault.resource_address();

    // Check that the transaction rejects if the fee paid is too low
    test.execute_expect_failure(
        Transaction::builder()
            .call_method(account, "withdraw", args![XTR2, Amount(9)])
            .put_last_instruction_output_on_workspace("fee")
            .call_method(
                component_address,
                "withdraw",
                args![Workspace("fee"), Amount(100)],
            )
            .put_last_instruction_output_on_workspace("bucket")
            .call_method(account, "deposit", args![Workspace("bucket")])
            .call_method(account, "balance", args![fungible_resource])
            .sign(&account_secret)
            .build(),
        vec![account_proof.clone()],
    );

    // Transfer some coins into the account
    let result = test.execute_expect_success(
        Transaction::builder()
            .call_method(account, "withdraw", args![XTR2, Amount(10)])
            .put_last_instruction_output_on_workspace("fee")
            .call_method(
                component_address,
                "withdraw",
                args![Workspace("fee"), Amount(100)],
            )
            .put_last_instruction_output_on_workspace("bucket")
            .call_method(account, "deposit", args![Workspace("bucket")])
            .call_method(account, "balance", args![fungible_resource])
            .sign(&account_secret)
            .build(),
        vec![account_proof],
    );

    let balance = result.expect_return::<Amount>(5);
    assert_eq!(balance, 100i64);

    let vault_id: VaultId = test.extract_component_value(component_address, "$.fee_vault");
    let vault = test.read_only_state_store().get_vault(&vault_id).unwrap();
    assert_eq!(vault.balance(), 10i64);

    let counter: u32 = test.extract_component_value(component_address, "$.counter");
    assert_eq!(counter, 1);
}

#[test]
fn exercise_5_nft_resource() {
    let mut test = TemplateTest::new(["."]);

    // Construct the component
    let (output, _, _) = generate_confidential_proof(INITIAL_SUPPLY, None);
    let component_address: ComponentAddress =
        test.call_function("Monerokon", "new", args![INITIAL_SUPPLY, output], vec![]);

    let vault_id: VaultId = test.extract_component_value(component_address, "$.nft_vault");
    let vault = test.read_only_state_store().get_vault(&vault_id).unwrap();
    assert_eq!(vault.resource_type(), ResourceType::NonFungible);
    assert_eq!(vault.balance(), 2i64);
}

#[test]
fn exercise_6_confidential() {
    let mut test = TemplateTest::new(["."]);

    // Construct the component
    let (output, initial_supply_bf, _) = generate_confidential_proof(INITIAL_SUPPLY, None);
    let component_address: ComponentAddress =
        test.call_function("Monerokon", "new", args![INITIAL_SUPPLY, output], vec![]);

    let (account, account_proof, account_secret) = test.create_funded_account();

    let vault_id: VaultId = test.extract_component_value(component_address, "$.confidential_vault");
    let vault = test.read_only_state_store().get_vault(&vault_id).unwrap();
    assert_eq!(
        vault.resource_type(),
        ResourceType::Confidential,
        "confidential_vault is not confidential"
    );

    let WithdrawProofOutput {
        proof: withdraw_proof,
        ..
    } = generate_withdraw_proof_with_inputs(
        &[(initial_supply_bf, INITIAL_SUPPLY)],
        Amount::zero(),
        Amount(1000),
        Some(INITIAL_SUPPLY - Amount(1000)),
        Amount::zero(),
    );

    // Transfer some coins into the account
    test.execute_expect_success(
        Transaction::builder()
            // Withdraw revealed fee funds
            .call_method(account, "withdraw", args![XTR2, Amount(10)])
            .put_last_instruction_output_on_workspace("fee")
            // Run withdraw_confidential on the component
            .call_method(
                component_address,
                "withdraw_confidential",
                args![Workspace("fee"), withdraw_proof],
            )
            .put_last_instruction_output_on_workspace("bucket")
            .call_method(account, "deposit", args![Workspace("bucket")])
            .sign(&account_secret)
            .build(),
        vec![account_proof],
    );
}

#[test]
fn exercise_7_mint() {
    let mut test = TemplateTest::new(["."]);

    // Construct the component
    let (output, _, _) = generate_confidential_proof(INITIAL_SUPPLY, None);
    let component_address: ComponentAddress =
        test.call_function("Monerokon", "new", args![INITIAL_SUPPLY, output], vec![]);

    // Get proof of ownership for the component.
    let proof = test.get_test_proof();

    let (output, _, _) = generate_confidential_proof(INITIAL_SUPPLY, None);

    // Transfer some coins into the account
    test.execute_expect_success(
        Transaction::builder()
            .call_method(component_address, "mint_fungible", args![Amount(1000)])
            .call_method(
                component_address,
                "mint_non_fungible",
                args![NonFungibleId::from_string("MoneroKon")],
            )
            .call_method(component_address, "mint_confidential", args![output])
            .sign(test.get_test_secret_key())
            .build(),
        vec![proof],
    );

    let supply_vault: VaultId = test.extract_component_value(component_address, "$.supply_vault");
    let supply_vault = test
        .read_only_state_store()
        .get_vault(&supply_vault)
        .unwrap();
    let nft_vault: VaultId = test.extract_component_value(component_address, "$.nft_vault");
    let nft_vault = test.read_only_state_store().get_vault(&nft_vault).unwrap();
    let confidential_vault: VaultId =
        test.extract_component_value(component_address, "$.confidential_vault");
    let confidential_vault = test
        .read_only_state_store()
        .get_vault(&confidential_vault)
        .unwrap();

    assert_eq!(supply_vault.balance(), INITIAL_SUPPLY + 1000);
    assert_eq!(nft_vault.balance(), 3i64);
    assert_eq!(
        confidential_vault
            .get_confidential_commitments()
            .unwrap()
            .len(),
        2
    );
}
