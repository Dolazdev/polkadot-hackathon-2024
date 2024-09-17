#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod event_manager {
    use ink_env::call::FromAccountId;
    use ink_storage::collections::{ HashMap, Vec as StorageVec };
    use ticket_nft::TicketNFT;

    #[ink(storage)]
    pub struct EventManager {
        owner: AccountId,
        next_event_id: u64,
        events: HashMap<u64, Event>,
        user_registered_events: HashMap<AccountId, StorageVec<u64>>,
    }

    #[derive(scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct EventDetails {
        title: String,
        date: String,
        location: String,
        ticket_price: u128,
        max_tickets: u64,
    }

    #[derive(scale::Encode, scale::Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Event {
        event_id: u64,
        details: EventDetails,
        ticket_nft_address: AccountId,
        attendees: StorageVec<AccountId>,
        tickets_sold: u64,
        active: bool,
        host: AccountId,
    }

    impl EventManager {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                owner: Self::env().caller(),
                next_event_id: 1,
                events: HashMap::new(),
                user_registered_events: HashMap::new(),
            }
        }

        #[ink(message)]
        pub fn create_event(
            &mut self,
            details: EventDetails,
            ticket_nft_address: AccountId
        ) -> u64 {
            let event_id = self.next_event_id;
            self.next_event_id += 1;

            let event = Event {
                event_id,
                details,
                ticket_nft_address,
                attendees: StorageVec::new(),
                tickets_sold: 0,
                active: true,
                host: self.env().caller(),
            };

            self.events.insert(event_id, event);
            event_id
        }

        #[ink(message, payable)]
        pub fn purchase_ticket(&mut self, event_id: u64, token_uri: String) -> bool {
            let event = match self.events.get_mut(&event_id) {
                Some(e) => e,
                None => {
                    return false;
                }
            };

            let caller = self.env().caller();
            let payment = self.env().transferred_balance();

            if
                event.active &&
                event.tickets_sold < event.details.max_tickets &&
                payment >= event.details.ticket_price
            {
                let mut nft_contract: TicketNFT = FromAccountId::from_account_id(
                    event.ticket_nft_address
                );
                let token_id = nft_contract.mint_ticket(caller, token_uri);

                if minted_ticket_id == 0 {
                    return false;
                }

                event.attendees.push(caller);
                event.tickets_sold += 1;

                let user_events = self.user_registered_events
                    .entry(caller)
                    .or_insert(StorageVec::new());
                user_events.push(event_id);

                true
            } else {
                false
            }
        }

        #[ink(message)]
        pub fn deactivate_event(&mut self, event_id: u64) -> bool {
            let event = match self.events.get_mut(&event_id) {
                Some(e) => e,
                None => {
                    return false;
                }
            };

            if event.host == self.env().caller() {
                event.active = false;
                true
            } else {
                false
            }
        }

        #[ink(message)]
        pub fn get_event_details(&self, event_id: u64) -> Option<EventDetails> {
            self.events.get(&event_id).map(|e| e.details.clone())
        }

        #[ink(message)]
        pub fn get_event_attendees(&self, event_id: u64) -> Option<StorageVec<AccountId>> {
            self.events.get(&event_id).map(|e| e.attendees.clone())
        }

        #[ink(message)]
        pub fn get_ticket_nft_address(&self, event_id: u64) -> Option<AccountId> {
            self.events.get(&event_id).map(|e| e.ticket_nft_address)
        }

        #[ink(message)]
        pub fn get_registered_events(&self, user: AccountId) -> Option<StorageVec<u64>> {
            self.user_registered_events.get(&user).cloned()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_env::AccountId;
        use ink_storage::collections::Vec as StorageVec;
        use ink_lang as ink;

        #[ink::test]
        fn test_create_event() {
            let mut contract = EventManager::new();
            let details = EventDetails {
                title: "Concert".to_string(),
                date: "2024-12-01".to_string(),
                location: "Stadium".to_string(),
                ticket_price: 1_000_000,
                max_tickets: 100,
            };

            let ticket_nft_address = AccountId::from([0x0; 32]); // Mock NFT address for testing
            let event_id = contract.create_event(details, ticket_nft_address);

            let event_details = contract.get_event_details(event_id);
            assert!(event_details.is_some());
            let event_details = event_details.unwrap();
            assert_eq!(event_details.title, "Concert");
            assert_eq!(event_details.date, "2024-12-01");
            assert_eq!(event_details.location, "Stadium");
            assert_eq!(event_details.ticket_price, 1_000_000);
            assert_eq!(event_details.max_tickets, 100);
        }

        #[ink::test]
        fn test_purchase_ticket() {
            let mut contract = EventManager::new();
            let details = EventDetails {
                title: "Concert".to_string(),
                date: "2024-12-01".to_string(),
                location: "Stadium".to_string(),
                ticket_price: 1_000_000,
                max_tickets: 100,
            };

            let ticket_nft_address = AccountId::from([0x0; 32]); // Mock NFT address for testing
            let event_id = contract.create_event(details, ticket_nft_address);

            // Attempting to purchase a ticket without sending any balance should fail
            let result = contract.purchase_ticket(event_id, "TicketURI".to_string());
            assert!(!result); // Should fail because no payment was made
        }

        #[ink::test]
        fn test_deactivate_event() {
            let mut contract = EventManager::new();
            let details = EventDetails {
                title: "Concert".to_string(),
                date: "2024-12-01".to_string(),
                location: "Stadium".to_string(),
                ticket_price: 1_000_000,
                max_tickets: 100,
            };

            let ticket_nft_address = AccountId::from([0x0; 32]); // Mock NFT address for testing
            let event_id = contract.create_event(details, ticket_nft_address);

            // Ensure the event is active
            let event = contract.get_event_details(event_id).unwrap();
            assert!(event.active);

            // Deactivate the event
            let result = contract.deactivate_event(event_id);
            assert!(result);

            // Verify the event is deactivated
            let event = contract.get_event_details(event_id).unwrap();
            assert!(!event.active);
        }

        #[ink::test]
        fn test_get_event_attendees() {
            let mut contract = EventManager::new();
            let details = EventDetails {
                title: "Concert".to_string(),
                date: "2024-12-01".to_string(),
                location: "Stadium".to_string(),
                ticket_price: 1_000_000,
                max_tickets: 100,
            };

            let ticket_nft_address = AccountId::from([0x0; 32]); // Mock NFT address for testing
            let event_id = contract.create_event(details, ticket_nft_address);

            let result = contract.purchase_ticket(event_id, "TicketURI".to_string());
            assert!(result);

            // Retrieve the attendees
            let attendees = contract.get_event_attendees(event_id);
            assert!(attendees.is_some());
            let attendees = attendees.unwrap();
            assert_eq!(attendees.len(), 1); // Expect one attendee (the contract caller)
        }

        #[ink::test]
        fn test_get_registered_events() {
            let mut contract = EventManager::new();
            let details = EventDetails {
                title: "Concert".to_string(),
                date: "2024-12-01".to_string(),
                location: "Stadium".to_string(),
                ticket_price: 1_000_000,
                max_tickets: 100,
            };

            let ticket_nft_address = AccountId::from([0x0; 32]); // Mock NFT address for testing
            let event_id = contract.create_event(details, ticket_nft_address);

            let result = contract.purchase_ticket(event_id, "TicketURI".to_string());
            assert!(result);

            // Retrieve the registered events for the caller
            let user = contract.env().caller();
            let registered_events = contract.get_registered_events(user);
            assert!(registered_events.is_some());
            let registered_events = registered_events.unwrap();
            assert_eq!(registered_events.len(), 1); // Expect one registered event
            assert_eq!(registered_events[0], event_id);
        }
    }
}
