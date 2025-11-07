#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Address, Symbol, String, Vec, symbol_short};

// Structure to store individual transaction details
#[contracttype]
#[derive(Clone)]
pub struct Transaction {
    pub tx_id: u64,
    pub from: Address,
    pub to: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub tx_type: String,  // "send", "receive", "contract_call"
    pub description: String,
}

// For creating unique transaction IDs
const TX_COUNTER: Symbol = symbol_short!("TX_COUNT");

// Mapping user address to their transaction history
#[contracttype]
pub enum TxHistory {
    UserTxs(Address),
}

#[contract]
pub struct TransactionHistoryContract;

#[contractimpl]
impl TransactionHistoryContract {
    
    /// Record a new transaction in the history
    pub fn record_transaction(
        env: Env,
        from: Address,
        to: Address,
        amount: i128,
        tx_type: String,
        description: String,
    ) -> u64 {
        // Authenticate the caller
        from.require_auth();
        
        // Get and increment transaction counter
        let mut tx_counter: u64 = env.storage().instance().get(&TX_COUNTER).unwrap_or(0);
        tx_counter += 1;
        
        // Get current timestamp
        let timestamp = env.ledger().timestamp();
        
        // Create new transaction record
        let new_tx = Transaction {
            tx_id: tx_counter,
            from: from.clone(),
            to: to.clone(),
            amount,
            timestamp,
            tx_type,
            description,
        };
        
        // Get existing transaction history for sender
        let sender_key = TxHistory::UserTxs(from.clone());
        let mut sender_txs: Vec<Transaction> = env.storage()
            .instance()
            .get(&sender_key)
            .unwrap_or(Vec::new(&env));
        
        // Add new transaction to sender's history
        sender_txs.push_back(new_tx.clone());
        env.storage().instance().set(&sender_key, &sender_txs);
        
        // Get existing transaction history for receiver
        let receiver_key = TxHistory::UserTxs(to.clone());
        let mut receiver_txs: Vec<Transaction> = env.storage()
            .instance()
            .get(&receiver_key)
            .unwrap_or(Vec::new(&env));
        
        // Add new transaction to receiver's history
        receiver_txs.push_back(new_tx);
        env.storage().instance().set(&receiver_key, &receiver_txs);
        
        // Update transaction counter
        env.storage().instance().set(&TX_COUNTER, &tx_counter);
        
        // Extend storage TTL
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Transaction recorded with ID: {}", tx_counter);
        tx_counter
    }
    
    /// Get complete transaction history for a user
    pub fn get_user_transactions(env: Env, user: Address) -> Vec<Transaction> {
        let key = TxHistory::UserTxs(user.clone());
        
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or(Vec::new(&env))
    }
    
    /// Get filtered transactions by type (e.g., "send", "receive")
    pub fn get_transactions_by_type(env: Env, user: Address, tx_type: String) -> Vec<Transaction> {
        let all_txs = Self::get_user_transactions(env.clone(), user);
        let mut filtered_txs = Vec::new(&env);
        
        for tx in all_txs.iter() {
            if tx.tx_type == tx_type {
                filtered_txs.push_back(tx);
            }
        }
        
        filtered_txs
    }

    pub fn get_total_transaction_count(env: Env) -> u64 {
        env.storage().instance().get(&TX_COUNTER).unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{Address, Env};

    #[test]
    fn test_record_and_retrieve_transaction() {
        let env = Env::default();
        let contract_id = env.register_contract(None, TransactionHistoryContract);
        let client = TransactionHistoryContractClient::new(&env, &contract_id);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        env.mock_all_auths();

        // Record a transaction
        let tx_id = client.record_transaction(
            &user1,
            &user2,
            &1000,
            &String::from_str(&env, "send"),
            &String::from_str(&env, "Payment for services"),
        );

        assert_eq!(tx_id, 1);


        let user1_txs = client.get_user_transactions(&user1);
        assert_eq!(user1_txs.len(), 1);

        let user2_txs = client.get_user_transactions(&user2);
        assert_eq!(user2_txs.len(), 1);
    }
}