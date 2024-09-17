#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod ticket_nft {
    use ink_storage::collections::HashMap;

    #[ink(storage)]
    pub struct TicketNFT {
        owner: AccountId,
        name: String,
        symbol: String,
        token_id_counter: u64,
        tokens: HashMap<u64, AccountId>, // Maps token_id to the owner
        token_uris: HashMap<u64, String>, // Maps token_id to a URI
    }

    impl TicketNFT {
        #[ink(constructor)]
        pub fn new(name: String, symbol: String) -> Self {
            Self {
                owner: Self::env().caller(),
                name,
                symbol,
                token_id_counter: 1,
                tokens: HashMap::new(),
                token_uris: HashMap::new(),
            }
        }

        #[ink(message)]
        pub fn mint_ticket(&mut self, recipient: AccountId, token_uri: String) -> u64 {
            let token_id = self.token_id_counter;
            self.token_id_counter += 1;
            self.tokens.insert(token_id, recipient);
            self.token_uris.insert(token_id, token_uri);
            token_id
        }

        #[ink(message)]
        pub fn get_owner_of(&self, token_id: u64) -> Option<AccountId> {
            self.tokens.get(&token_id).copied()
        }

        #[ink(message)]
        pub fn get_token_uri(&self, token_id: u64) -> Option<String> {
            self.token_uris.get(&token_id).cloned()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn test_mint_ticket() {
            let mut nft_contract = TicketNFT::new("BlockPassNFT".to_string(), "BPNT".to_string());
            let recipient = AccountId::from([0x1; 32]);
            let token_uri = "https://example.com/nft/1".to_string();
            let token_id = nft_contract.mint_ticket(recipient, token_uri.clone());
            assert_eq!(nft_contract.get_owner_of(token_id), Some(recipient));
            assert_eq!(nft_contract.get_token_uri(token_id), Some(token_uri));
        }
    }
}
