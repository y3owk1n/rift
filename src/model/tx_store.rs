use std::sync::Arc;

use dashmap::DashMap;
use dashmap::mapref::entry::Entry;
use objc2_core_foundation::CGRect;

use crate::actor::reactor::transaction_manager::TransactionId;
use crate::sys::window_server::WindowServerId;

#[derive(Clone, Copy, Debug, Default)]
pub struct TxRecord {
    pub txid: TransactionId,
    pub target: Option<CGRect>,
}

/// Thread-safe cache mapping window server IDs to their last known transaction.
#[derive(Clone, Default, Debug)]
pub struct WindowTxStore(Arc<DashMap<WindowServerId, TxRecord>>);

impl WindowTxStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, id: WindowServerId, txid: TransactionId, target: CGRect) {
        match self.0.entry(id) {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() = TxRecord { txid, target: Some(target) };
            }
            Entry::Vacant(entry) => {
                entry.insert(TxRecord { txid, target: Some(target) });
            }
        }
    }

    pub fn get(&self, id: &WindowServerId) -> Option<TxRecord> {
        self.0.get(id).map(|entry| *entry)
    }

    pub fn remove(&self, id: &WindowServerId) {
        self.0.remove(id);
    }

    pub fn next_txid(&self, id: WindowServerId) -> TransactionId {
        match self.0.entry(id) {
            Entry::Occupied(mut entry) => {
                let record = entry.get_mut();
                let new_txid = record.txid.next();
                *record = TxRecord { txid: new_txid, target: None };
                new_txid
            }
            Entry::Vacant(entry) => {
                let txid = TransactionId::default().next();
                entry.insert(TxRecord { txid, target: None });
                txid
            }
        }
    }

    pub fn set_last_txid(&self, id: WindowServerId, txid: TransactionId) {
        match self.0.entry(id) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().txid = txid;
            }
            Entry::Vacant(entry) => {
                entry.insert(TxRecord { txid, target: None });
            }
        }
    }

    pub fn last_txid(&self, id: &WindowServerId) -> TransactionId {
        self.get(id).map(|record| record.txid).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use objc2_core_foundation::{CGPoint, CGSize};

    use super::*;

    #[test]
    fn test_tx_record_default() {
        let record = TxRecord::default();
        assert_eq!(record.txid, TransactionId::default());
        assert_eq!(record.target, None);
    }

    #[test]
    fn test_window_tx_store_insert_and_get() {
        let store = WindowTxStore::new();
        let wsid = WindowServerId::new(1);
        let txid = store.next_txid(wsid);
        let frame = CGRect::new(CGPoint::new(100.0, 100.0), CGSize::new(400.0, 300.0));

        store.insert(wsid, txid, frame);
        let record = store.get(&wsid).expect("Should have record");
        assert_eq!(record.txid, txid);
        assert_eq!(record.target, Some(frame));
    }

    #[test]
    fn test_window_tx_store_remove() {
        let store = WindowTxStore::new();
        let wsid = WindowServerId::new(1);
        let txid = store.next_txid(wsid);

        store.insert(wsid, txid, CGRect::ZERO);
        assert!(store.get(&wsid).is_some());

        store.remove(&wsid);
        assert!(store.get(&wsid).is_none());
    }

    #[test]
    fn test_window_tx_store_next_txid() {
        let store = WindowTxStore::new();
        let wsid = WindowServerId::new(1);

        let txid1 = store.next_txid(wsid);
        let txid2 = store.next_txid(wsid);
        let txid3 = store.next_txid(wsid);

        assert_ne!(txid1, txid2);
        assert_ne!(txid2, txid3);
        assert_ne!(txid1, txid3);
    }

    #[test]
    fn test_window_tx_store_set_last_txid() {
        let store = WindowTxStore::new();
        let wsid = WindowServerId::new(1);

        let txid = store.next_txid(wsid);
        store.set_last_txid(wsid, txid);
        assert_eq!(store.last_txid(&wsid), txid);
    }

    #[test]
    fn test_window_tx_store_get_nonexistent() {
        let store = WindowTxStore::new();
        let wsid = WindowServerId::new(999);

        assert!(store.get(&wsid).is_none());
        assert_eq!(store.last_txid(&wsid), TransactionId::default());
    }

    #[test]
    fn test_window_tx_store_overwrite() {
        let store = WindowTxStore::new();
        let wsid = WindowServerId::new(1);
        let frame1 = CGRect::new(CGPoint::new(100.0, 100.0), CGSize::new(400.0, 300.0));
        let frame2 = CGRect::new(CGPoint::new(200.0, 200.0), CGSize::new(500.0, 400.0));

        let txid1 = store.next_txid(wsid);
        store.insert(wsid, txid1, frame1);
        let record1 = store.get(&wsid).unwrap();
        assert_eq!(record1.target, Some(frame1));

        let txid2 = store.next_txid(wsid);
        store.insert(wsid, txid2, frame2);
        let record2 = store.get(&wsid).unwrap();
        assert_ne!(record2.txid, record1.txid);
        assert_eq!(record2.target, Some(frame2));
    }

    #[test]
    fn test_window_tx_store_clone() {
        let store = WindowTxStore::new();
        let wsid = WindowServerId::new(1);
        let txid = store.next_txid(wsid);

        store.insert(wsid, txid, CGRect::ZERO);
        let cloned = store.clone();
        let record = cloned.get(&wsid).expect("Should exist in cloned store");
        assert_eq!(record.txid, txid);
    }
}
