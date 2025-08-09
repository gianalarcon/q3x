use candid::{CandidType, Principal};
//use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use ic_ledger_types::{
    AccountIdentifier, BlockIndex, Memo, Subaccount, Tokens, DEFAULT_SUBACCOUNT,
    MAINNET_LEDGER_CANISTER_ID,
};

use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct TransferArgs {
    pub(crate) amount: Tokens,
    pub(crate) to_principal: Principal,
    pub(crate) to_subaccount: Option<Subaccount>,
}
#[derive(Debug, PartialEq)]
pub enum WalletError {
    /// Represents an error when the signature provided is invalid.
    InvalidSignature,
    /// Error when a message is already queued for signing.
    MsgAlreadyQueued,
    /// Error when a message is not found in the queue.
    MsgNotQueued,
    /// Error when there are not enough signers to meet the threshold.
    NotEnoughSigners,
}

/// A trait defining the behaviors of a MultiSignature Wallet.
pub trait MultiSignatureWallet {
    /// Add a new signer to the wallet.
    ///
    /// * `signer` - The `Principal` of the signer to add.
    fn add_signer(&mut self, signer: Principal);

    /// Remove an existing signer from the wallet.
    ///
    /// * `signer` - The `Principal` of the signer to remove.
    fn remove_signer(&mut self, signer: Principal);

    /// Get a list of all current signers of the wallet.
    ///
    /// Returns a `Vec<Principal>` containing the principals of all signers.
    fn get_signers(&self) -> Vec<Principal>;

    /// Set the default threshold for signing.
    ///
    /// * `threshold` - The threshold as a `u8` value.
    ///
    /// Returns `Result<(), WalletError>` indicating success or the type of failure.
    fn set_default_threshold(&mut self, threshold: u8) -> Result<(), WalletError>;

    /// Transfer tokens to a principal.
    ///
    /// * `args` - The `TransferArgs` struct containing the amount, to_principal, and to_subaccount.
    ///
    /// Returns `Result<BlockIndex, String>` indicating success or the type of failure.
    async fn transfer(&self, args: TransferArgs) -> Result<BlockIndex, String>;

    /// Check if a given `Principal` is a signer in the wallet.
    ///
    /// * `signer` - The `Principal` to check.
    ///
    /// Returns `bool` indicating whether the signer is present.
    fn has_signer(&self, signer: Principal) -> bool;

    /// Get the current default threshold for signing.
    ///
    /// Returns the threshold as a `u8` value.
    fn get_default_threshold(&self) -> u8;

    /// Propose a new message for signing.
    ///
    /// * `caller` - The `Principal` proposing the message.
    /// * `msg` - The message as a `Vec<u8>`.
    ///
    /// Returns `Result<(), WalletError>` indicating success or the type of failure.
    fn propose_message(&mut self, caller: Principal, msg: Vec<u8>) -> Result<(), WalletError>;

    /// Check if a message can be signed according to the current rules.
    ///
    /// * `msg` - A reference to the message as a `Vec<u8>`.
    ///
    /// Returns `bool` indicating whether the message can be signed.
    fn can_sign(&self, msg: &Vec<u8>) -> bool;

    /// Approve a message with a signer's consent.
    ///
    /// * `msg` - The message as a `Vec<u8>`.
    /// * `signer` - The `Principal` of the signer approving the message.
    ///
    /// Returns `Result<u8, WalletError>` indicating the number of approvals or the type of failure.
    fn approve(&mut self, msg: Vec<u8>, signer: Principal) -> Result<u8, WalletError>;

    /// Returns all messages that can be signed.
    ///
    /// Returns a `Vec<Vec<u8>>` containing the messages that can be signed.
    fn get_messages_to_sign(&self) -> Vec<Vec<u8>>;

    /// Returns all messages that have been proposed.
    ///
    /// Returns a `Vec<Vec<u8>>` containing the messages that have been proposed.
    fn get_proposed_messages(&self) -> Vec<Vec<u8>>;

    /// Returns all messages that have been proposed with their signers.
    ///
    /// Returns a `Vec<(Vec<u8>, Vec<Principal>)>` containing the messages that have been proposed with their signers.
    fn get_messages_with_signers(&self) -> Vec<(Vec<u8>, Vec<Principal>)>;

    /// Add metadata to a message in the wallet.
    ///
    /// * `message` - The message as a `Vec<u8>`.
    /// * `metadata` - The metadata as a `String`.
    /// * `caller` - The `Principal` of the caller.
    ///
    /// Returns `Result<(), String>` indicating success or the type of failure.
    fn add_metadata(
        &mut self,
        message: Vec<u8>,
        metadata: String,
        caller: Principal,
    ) -> Result<(), String>;

    /// Get the metadata associated with a message in the wallet.
    ///
    /// * `message` - The message as a `Vec<u8>`.
    /// * `caller` - The `Principal` of the caller.
    ///
    /// Returns `Option<&String>` containing the metadata if it exists.
    fn get_metadata(&self, message: Vec<u8>, caller: Principal) -> Option<&String>;

    /// Remove a message and its metadata from the wallet.
    ///
    /// * `msg` - The message as a `Vec<u8>`.
    /// * `caller` - The `Principal` of the caller.
    ///
    /// Returns `Result<(), String>` indicating success or the type of failure.
    fn remove_message_and_metadata(
        &mut self,
        msg: Vec<u8>,
        caller: Principal,
    ) -> Result<(), String>;
}

#[derive(Clone, Debug, CandidType, Deserialize, Default)]
pub struct Wallet {
    /// A set of signers for the wallet, represented by their `Principal`.
    signers: HashSet<Principal>,
    /// The threshold number of signers required for certain actions.
    threshold: u8,
    /// A map tracking messages and the list of signers who have already signed them.
    message_queue: HashMap<Vec<u8>, Vec<Principal>>,
    /// A map tracking messages and their metadata.
    metadata: HashMap<Vec<u8>, String>,
}

impl MultiSignatureWallet for Wallet {
    fn add_signer(&mut self, signer: Principal) {
        self.signers.insert(signer);
    }

    fn remove_signer(&mut self, signer: Principal) {
        self.signers.remove(&signer);
    }

    fn get_signers(&self) -> Vec<Principal> {
        self.signers.iter().cloned().collect()
    }

    fn set_default_threshold(&mut self, threshold: u8) -> Result<(), WalletError> {
        if self.signers.len() < threshold as usize {
            return Err(WalletError::NotEnoughSigners);
        }
        self.threshold = threshold;
        Ok(())
    }

    // transfer icp token
    async fn transfer(&self, args: TransferArgs) -> Result<BlockIndex, String> {
        ic_cdk::println!(
            "Transferring {} tokens to principal {} subaccount {:?}",
            &args.amount,
            &args.to_principal,
            &args.to_subaccount
        );
        let to_subaccount = args.to_subaccount.unwrap_or(DEFAULT_SUBACCOUNT);
        let transfer_args = ic_ledger_types::TransferArgs {
            memo: Memo(0),
            amount: args.amount,
            fee: Tokens::from_e8s(10_000),
            // The subaccount of the account identifier that will be used to withdraw tokens and send them
            // to another account identifier. If set to None then the default subaccount will be used.
            // See the [Ledger doc](https://internetcomputer.org/docs/current/developer-docs/integrations/ledger/#accounts).
            from_subaccount: None,
            to: AccountIdentifier::new(&args.to_principal, &to_subaccount),
            created_at_time: None,
        };
        ic_ledger_types::transfer(MAINNET_LEDGER_CANISTER_ID, transfer_args)
            .await
            .map_err(|e| format!("failed to call ledger: {e:?}"))?
            .map_err(|e| format!("ledger transfer error {e:?}"))
    }

    fn has_signer(&self, signer: Principal) -> bool {
        self.signers.contains(&signer)
    }

    fn get_default_threshold(&self) -> u8 {
        self.threshold
    }

    fn propose_message(&mut self, caller: Principal, msg: Vec<u8>) -> Result<(), WalletError> {
        if !self.signers.contains(&caller) {
            return Err(WalletError::InvalidSignature);
        }

        if self.message_queue.contains_key(&msg) {
            return Err(WalletError::MsgAlreadyQueued);
        }

        self.message_queue.insert(msg.clone(), Vec::new());

        Ok(())
    }

    fn can_sign(&self, msg: &Vec<u8>) -> bool {
        if !self.message_queue.contains_key(msg) {
            return false;
        }
        self.message_queue[msg].len() >= self.threshold as usize
    }

    fn approve(&mut self, msg: Vec<u8>, signer: Principal) -> Result<u8, WalletError> {
        if !self.message_queue.contains_key(&msg) {
            return Err(WalletError::MsgNotQueued);
        }

        if !self.signers.contains(&signer) {
            return Err(WalletError::InvalidSignature);
        }

        let queue = self.message_queue.get_mut(&msg).unwrap();
        queue.push(signer);

        Ok(queue.len() as u8)
    }

    fn get_messages_to_sign(&self) -> Vec<Vec<u8>> {
        self.message_queue
            .iter()
            .filter(|(msg, _)| self.can_sign(msg))
            .map(|(msg, _)| msg.clone())
            .collect()
    }

    fn get_proposed_messages(&self) -> Vec<Vec<u8>> {
        self.message_queue.keys().cloned().collect()
    }

    fn get_messages_with_signers(&self) -> Vec<(Vec<u8>, Vec<Principal>)> {
        self.message_queue
            .iter()
            .map(|(msg, signers)| (msg.clone(), signers.clone()))
            .collect()
    }

    fn add_metadata(
        &mut self,
        message: Vec<u8>,
        metadata: String,
        caller: Principal,
    ) -> Result<(), String> {
        if !self.signers.contains(&caller) {
            return Err("Cannot add metadata: No signer.".to_string());
        }
        if !self.message_queue.contains_key(&message) {
            return Err("Cannot add metadata: Message not found.".to_string());
        }
        if self.metadata.contains_key(&message) {
            return Err("Metadata already exists for this message".to_string());
        }
        self.metadata.insert(message, metadata);
        Ok(())
    }

    fn get_metadata(&self, message: Vec<u8>, caller: Principal) -> Option<&String> {
        if !self.signers.contains(&caller) {
            return None;
        }
        self.metadata.get(&message)
    }

    fn remove_message_and_metadata(
        &mut self,
        msg: Vec<u8>,
        caller: Principal,
    ) -> Result<(), String> {
        // Check if the caller is a signer
        if !self.signers.contains(&caller) {
            return Err("CallerNotSigner".to_string());
        }

        // Remove the message and its metadata
        self.message_queue.remove(&msg);
        self.metadata.remove(&msg);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;
    use std::str::FromStr;

    #[test]
    fn test_default_wallet() {
        let wallet = Wallet::default();
        assert_eq!(wallet.get_signers().len(), 0);
        assert_eq!(wallet.get_default_threshold(), 0);
    }

    #[test]
    fn test_add_remove_signer() {
        let mut wallet = Wallet::default();

        let signer1 = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        let signer2 = Principal::from_str("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

        wallet.add_signer(signer1);
        wallet.add_signer(signer2);

        let signers = wallet.get_signers();
        assert_eq!(signers.len(), 2);
        assert!(signers.contains(&signer1));
        assert!(signers.contains(&signer2));

        wallet.remove_signer(signer1);
        let signers = wallet.get_signers();
        assert_eq!(signers.len(), 1);
        assert!(!signers.contains(&signer1));
        assert!(signers.contains(&signer2));
    }

    #[test]
    fn test_set_get_default_threshold() {
        let mut wallet = Wallet::default();

        assert_eq!(wallet.get_default_threshold(), 0);

        wallet.add_signer(Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap());
        let _ = wallet.set_default_threshold(1);
        assert_eq!(wallet.get_default_threshold(), 1);
        assert_eq!(
            wallet.set_default_threshold(2),
            Err(WalletError::NotEnoughSigners)
        );
    }

    #[test]
    fn test_propose_message_valid_signature() {
        let mut wallet = Wallet::default();
        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        wallet.add_signer(signer);

        let msg = vec![1, 2, 3];
        let result = wallet.propose_message(signer, msg);

        assert!(result.is_ok());
    }

    #[test]
    fn test_propose_message_invalid_signature() {
        let mut wallet = Wallet::default();
        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        // Don't add the signer to the wallet.
        // wallet.add_signer(signer.clone());

        let msg = vec![1, 2, 3];
        let result = wallet.propose_message(signer, msg.clone());

        assert_eq!(result.err(), Some(WalletError::InvalidSignature));
    }

    #[test]
    fn test_propose_message_duplicate_message() {
        let mut wallet = Wallet::default();
        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        wallet.add_signer(signer);

        let msg = vec![1, 2, 3];
        let _ = wallet.propose_message(signer, msg.clone());

        // Try proposing the same message again.
        let result = wallet.propose_message(signer, msg);

        assert_eq!(result.err(), Some(WalletError::MsgAlreadyQueued));
    }

    #[test]
    fn test_can_sign_threshold_not_met() {
        let mut wallet = Wallet::default();

        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        wallet.add_signer(signer);
        let _ = wallet.set_default_threshold(1);

        let msg = vec![1, 2, 3];
        let _ = wallet.propose_message(signer, msg.clone());

        // Threshold is not met, so cannot sign.
        let can_sign = wallet.can_sign(&msg.clone());

        assert!(!can_sign);
    }

    #[test]
    fn test_can_sign_message_not_queued() {
        let mut wallet = Wallet::default();
        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        wallet.add_signer(signer);

        let msg = vec![1, 2, 3];

        // Message is not in the queue, so cannot sign.
        let can_sign = wallet.can_sign(&msg.clone());

        assert!(!can_sign);
    }

    #[test]
    fn test_approve_valid_message() {
        let mut wallet = Wallet::default();
        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        wallet.add_signer(signer);
        let _ = wallet.set_default_threshold(1);

        let msg = vec![1, 2, 3];
        wallet.propose_message(signer, msg.clone()).unwrap();

        assert!(!wallet.can_sign(&msg));
        let result = wallet.approve(msg.clone(), signer);

        assert!(result.is_ok());
        assert!(wallet.can_sign(&msg));
    }

    #[test]
    fn test_approve_invalid_message() {
        let mut wallet = Wallet::default();
        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        wallet.add_signer(signer);

        let msg = vec![1, 2, 3];

        let result = wallet.approve(msg.clone(), signer);

        assert_eq!(result.err(), Some(WalletError::MsgNotQueued));
    }

    #[test]
    fn test_approve_not_signer() {
        let mut wallet = Wallet::default();
        let signer1 = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        let signer2 = Principal::from_str("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        let _ = wallet.set_default_threshold(1);
        wallet.add_signer(signer1);

        let msg = vec![1, 2, 3];
        wallet.propose_message(signer1, msg.clone()).unwrap();

        let result = wallet.approve(msg.clone(), signer2);

        assert_eq!(result.err(), Some(WalletError::InvalidSignature));
    }

    #[test]
    fn test_has_signer() {
        let mut wallet = Wallet::default();
        let signer1 = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        let signer2 = Principal::from_str("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        wallet.add_signer(signer1);

        assert!(wallet.has_signer(signer1));
        assert!(!wallet.has_signer(signer2));
    }

    #[test]
    fn test_get_messages() {
        let mut wallet = Wallet::default();
        let signer1 = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        let signer2 = Principal::from_str("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        wallet.add_signer(signer1);
        wallet.add_signer(signer2);
        let _ = wallet.set_default_threshold(1);

        let msg1 = vec![1, 2, 3];
        let msg2 = vec![4, 5, 6];
        let msg3 = vec![7, 8, 9];
        wallet.propose_message(signer1, msg1.clone()).unwrap();
        wallet.propose_message(signer2, msg2.clone()).unwrap();
        wallet.propose_message(signer2, msg3.clone()).unwrap();

        wallet.approve(msg1.clone(), signer1).unwrap();
        wallet.approve(msg2.clone(), signer2).unwrap();

        let messages_to_sign = wallet.get_messages_to_sign();

        assert_eq!(messages_to_sign.len(), 2);
        assert!(messages_to_sign.contains(&msg1));
        assert!(messages_to_sign.contains(&msg2));

        let proposed_messages = wallet.get_proposed_messages();

        assert_eq!(proposed_messages.len(), 3);

        let all_messages_with_signers = wallet.get_messages_with_signers();

        assert_eq!(all_messages_with_signers.len(), 3);
        assert!(all_messages_with_signers.contains(&(msg1.clone(), vec![signer1])));
        assert!(all_messages_with_signers.contains(&(msg2.clone(), vec![signer2])));
        assert!(all_messages_with_signers.contains(&(msg3.clone(), vec![])));
    }

    #[test]
    fn test_add_metadata_valid() {
        let mut wallet = Wallet::default();
        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        wallet.add_signer(signer);

        let msg = vec![1, 2, 3];
        let _ = wallet.propose_message(signer, msg.clone());

        let result = wallet.add_metadata(msg.clone(), "metadata".to_string(), signer);

        assert!(result.is_ok());
        assert_eq!(
            wallet.get_metadata(msg.clone(), signer),
            Some(&"metadata".to_string())
        );
    }

    #[test]
    fn test_add_metadata_invalid_message() {
        let mut wallet = Wallet::default();
        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        wallet.add_signer(signer);

        let msg = vec![1, 2, 3];

        let result = wallet.add_metadata(msg.clone(), "metadata".to_string(), signer);

        assert_eq!(
            result.err(),
            Some("Cannot add metadata: Message not found.".to_string())
        );
    }

    #[test]
    fn test_add_metadata_invalid_signer() {
        let mut wallet = Wallet::default();
        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        let invalid_signer = Principal::from_str("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        wallet.add_signer(signer);

        let msg = vec![1, 2, 3];
        let _ = wallet.propose_message(signer, msg.clone());

        let result = wallet.add_metadata(msg.clone(), "metadata".to_string(), invalid_signer);

        assert_eq!(
            result.err(),
            Some("Cannot add metadata: No signer.".to_string())
        );
    }

    #[test]
    fn test_get_metadata_exists() {
        let mut wallet = Wallet::default();
        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        wallet.add_signer(signer);

        let msg = vec![1, 2, 3];
        let _ = wallet.propose_message(signer, msg.clone());
        let _ = wallet.add_metadata(msg.clone(), "metadata".to_string(), signer);

        let result = wallet.get_metadata(msg.clone(), signer);

        assert_eq!(result, Some(&"metadata".to_string()));
    }

    #[test]
    fn test_get_metadata_not_exists() {
        let mut wallet = Wallet::default();
        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        wallet.add_signer(signer);

        let msg = vec![1, 2, 3];
        let _ = wallet.propose_message(signer, msg.clone());

        let result = wallet.get_metadata(msg.clone(), signer);

        assert_eq!(result, None);
    }

    #[test]
    fn test_get_metadata_returns_none_if_not_signer() {
        let mut wallet = Wallet::default();
        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        let invalid_signer = Principal::from_str("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
        wallet.add_signer(signer);

        let msg = vec![1, 2, 3];
        let _ = wallet.propose_message(signer, msg.clone());
        let _ = wallet.add_metadata(msg.clone(), "metadata".to_string(), signer);

        let result = wallet.get_metadata(msg.clone(), invalid_signer);

        assert_eq!(result, None);
    }

    #[test]
    fn test_add_metadata_only_once() {
        let mut wallet = Wallet::default();
        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        wallet.add_signer(signer);

        let msg = vec![1, 2, 3];
        let _ = wallet.propose_message(signer, msg.clone());

        let result = wallet.add_metadata(msg.clone(), "metadata".to_string(), signer.clone());
        assert!(result.is_ok());
        assert_eq!(
            wallet.get_metadata(msg.clone(), signer),
            Some(&"metadata".to_string())
        );

        // Try to add metadata again to the same message
        let result = wallet.add_metadata(msg.clone(), "new metadata".to_string(), signer.clone());
        assert_eq!(
            result.err(),
            Some("Metadata already exists for this message".to_string())
        );
    }

    #[test]
    fn test_remove_message_and_metadata() {
        let mut wallet = Wallet::default();
        let signer = Principal::from_str("2chl6-4hpzw-vqaaa-aaaaa-c").unwrap();
        wallet.add_signer(signer);

        let msg = vec![1, 2, 3];
        let _ = wallet.propose_message(signer, msg.clone());
        let _ = wallet.add_metadata(msg.clone(), "metadata".to_string(), signer);

        assert_eq!(
            wallet.get_metadata(msg.clone(), signer),
            Some(&"metadata".to_string())
        );

        let result = wallet.remove_message_and_metadata(msg.clone(), signer);

        assert!(result.is_ok());
        assert_eq!(wallet.get_metadata(msg.clone(), signer), None);
        assert!(!wallet.message_queue.contains_key(&msg));
    }
}
