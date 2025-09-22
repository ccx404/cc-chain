//! Contract Event System
//!
//! This module provides event logging and filtering capabilities for smart contracts.

use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Contract event manager
#[derive(Debug, Default)]
pub struct EventManager {
    /// Event listeners by contract
    listeners: HashMap<String, Vec<EventListener>>,

    /// Event history
    event_log: Vec<ContractEvent>,

    /// Event filters
    #[allow(dead_code)]
    filters: Vec<EventFilter>,
}

/// Contract event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    /// Contract that emitted the event
    pub contract_address: String,

    /// Event name
    pub event_name: String,

    /// Event topics (indexed parameters)
    pub topics: Vec<Vec<u8>>,

    /// Event data (non-indexed parameters)
    pub data: Vec<u8>,

    /// Block number when event was emitted
    pub block_number: u64,

    /// Transaction hash that triggered the event
    pub transaction_hash: String,

    /// Event index in the transaction
    pub log_index: u32,

    /// Timestamp
    pub timestamp: u64,
}

/// Event listener for contract notifications
#[derive(Debug, Clone)]
pub struct EventListener {
    /// Listener ID
    pub id: String,

    /// Events to listen for
    pub event_filter: EventFilter,

    /// Callback for notifications
    pub callback: String, // In a real implementation, this would be a function pointer
}

/// Event filter for querying events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilter {
    /// Contract addresses to filter by
    pub addresses: Option<Vec<String>>,

    /// Event names to filter by
    pub event_names: Option<Vec<String>>,

    /// Topics to filter by
    pub topics: Option<Vec<Option<Vec<u8>>>>,

    /// Starting block number
    pub from_block: Option<u64>,

    /// Ending block number
    pub to_block: Option<u64>,

    /// Maximum number of events to return
    pub limit: Option<usize>,
}

/// Event query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventQueryResult {
    /// Matching events
    pub events: Vec<ContractEvent>,

    /// Whether there are more events available
    pub has_more: bool,

    /// Total number of matching events
    pub total_count: usize,
}

impl EventManager {
    /// Create a new event manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Emit an event from a contract
    pub fn emit_event(
        &mut self,
        contract_address: String,
        event_name: String,
        topics: Vec<Vec<u8>>,
        data: Vec<u8>,
        block_number: u64,
        transaction_hash: String,
        log_index: u32,
    ) -> Result<()> {
        let event = ContractEvent {
            contract_address: contract_address.clone(),
            event_name: event_name.clone(),
            topics,
            data,
            block_number,
            transaction_hash,
            log_index,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Add to event log
        self.event_log.push(event.clone());

        // Notify listeners
        if let Some(listeners) = self.listeners.get(&contract_address) {
            for listener in listeners {
                if self.matches_filter(&event, &listener.event_filter) {
                    // In a real implementation, this would trigger the callback
                    tracing::debug!("Event notification for listener {}", listener.id);
                }
            }
        }

        Ok(())
    }

    /// Register an event listener
    pub fn register_listener(
        &mut self,
        contract_address: String,
        listener: EventListener,
    ) -> Result<()> {
        self.listeners
            .entry(contract_address)
            .or_insert_with(Vec::new)
            .push(listener);

        Ok(())
    }

    /// Remove an event listener
    pub fn remove_listener(&mut self, contract_address: &str, listener_id: &str) -> Result<bool> {
        if let Some(listeners) = self.listeners.get_mut(contract_address) {
            let initial_len = listeners.len();
            listeners.retain(|l| l.id != listener_id);
            Ok(listeners.len() < initial_len)
        } else {
            Ok(false)
        }
    }

    /// Query events based on filter
    pub fn query_events(&self, filter: &EventFilter) -> Result<EventQueryResult> {
        let mut matching_events = Vec::new();

        for event in &self.event_log {
            if self.matches_filter(event, filter) {
                matching_events.push(event.clone());
            }
        }

        // Apply limit if specified
        let total_count = matching_events.len();
        let has_more = if let Some(limit) = filter.limit {
            if matching_events.len() > limit {
                matching_events.truncate(limit);
                true
            } else {
                false
            }
        } else {
            false
        };

        Ok(EventQueryResult {
            events: matching_events,
            has_more,
            total_count,
        })
    }

    /// Check if event matches filter
    fn matches_filter(&self, event: &ContractEvent, filter: &EventFilter) -> bool {
        // Check addresses
        if let Some(ref addresses) = filter.addresses {
            if !addresses.contains(&event.contract_address) {
                return false;
            }
        }

        // Check event names
        if let Some(ref event_names) = filter.event_names {
            if !event_names.contains(&event.event_name) {
                return false;
            }
        }

        // Check topics
        if let Some(ref filter_topics) = filter.topics {
            for (i, filter_topic) in filter_topics.iter().enumerate() {
                if let Some(ref expected_topic) = filter_topic {
                    if event.topics.get(i) != Some(expected_topic) {
                        return false;
                    }
                }
            }
        }

        // Check block range
        if let Some(from_block) = filter.from_block {
            if event.block_number < from_block {
                return false;
            }
        }

        if let Some(to_block) = filter.to_block {
            if event.block_number > to_block {
                return false;
            }
        }

        true
    }

    /// Get all events for a contract
    pub fn get_contract_events(&self, contract_address: &str) -> Vec<ContractEvent> {
        self.event_log
            .iter()
            .filter(|event| event.contract_address == contract_address)
            .cloned()
            .collect()
    }

    /// Get event count for a contract
    pub fn get_event_count(&self, contract_address: Option<&str>) -> usize {
        if let Some(address) = contract_address {
            self.event_log
                .iter()
                .filter(|event| event.contract_address == address)
                .count()
        } else {
            self.event_log.len()
        }
    }

    /// Clear old events (for storage management)
    pub fn prune_events(&mut self, before_block: u64) -> usize {
        let initial_len = self.event_log.len();
        self.event_log
            .retain(|event| event.block_number >= before_block);
        initial_len - self.event_log.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_emission() {
        let mut manager = EventManager::new();

        manager
            .emit_event(
                "contract123".to_string(),
                "Transfer".to_string(),
                vec![b"from".to_vec(), b"to".to_vec()],
                b"amount_data".to_vec(),
                100,
                "tx_hash".to_string(),
                0,
            )
            .unwrap();

        assert_eq!(manager.get_event_count(None), 1);
        assert_eq!(manager.get_event_count(Some("contract123")), 1);
        assert_eq!(manager.get_event_count(Some("contract456")), 0);
    }

    #[test]
    fn test_event_filtering() {
        let mut manager = EventManager::new();

        // Add test events
        manager
            .emit_event(
                "contract123".to_string(),
                "Transfer".to_string(),
                vec![b"alice".to_vec(), b"bob".to_vec()],
                b"100".to_vec(),
                100,
                "tx1".to_string(),
                0,
            )
            .unwrap();

        manager
            .emit_event(
                "contract456".to_string(),
                "Approval".to_string(),
                vec![b"alice".to_vec(), b"charlie".to_vec()],
                b"50".to_vec(),
                101,
                "tx2".to_string(),
                0,
            )
            .unwrap();

        // Test filtering by contract
        let filter = EventFilter {
            addresses: Some(vec!["contract123".to_string()]),
            event_names: None,
            topics: None,
            from_block: None,
            to_block: None,
            limit: None,
        };

        let result = manager.query_events(&filter).unwrap();
        assert_eq!(result.events.len(), 1);
        assert_eq!(result.events[0].contract_address, "contract123");
    }

    #[test]
    fn test_event_listener() {
        let mut manager = EventManager::new();

        let listener = EventListener {
            id: "listener1".to_string(),
            event_filter: EventFilter {
                addresses: Some(vec!["contract123".to_string()]),
                event_names: Some(vec!["Transfer".to_string()]),
                topics: None,
                from_block: None,
                to_block: None,
                limit: None,
            },
            callback: "handle_transfer".to_string(),
        };

        manager
            .register_listener("contract123".to_string(), listener)
            .unwrap();

        // Emit event that should trigger listener
        manager
            .emit_event(
                "contract123".to_string(),
                "Transfer".to_string(),
                vec![],
                vec![],
                100,
                "tx1".to_string(),
                0,
            )
            .unwrap();

        assert_eq!(manager.listeners.len(), 1);
    }
}
