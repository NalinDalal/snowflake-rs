use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Number of bits allocated to each part of the Snowflake ID
const SIGN_BITS: u64 = 1;
const TIMESTAMP_BITS: u64 = 41;
const DATACENTER_BITS: u64 = 5;
const MACHINE_BITS: u64 = 5;
const SEQUENCE_BITS: u64 = 12;

/// Bit shifts
const MACHINE_SHIFT: u64 = SEQUENCE_BITS;
const DATACENTER_SHIFT: u64 = MACHINE_SHIFT + MACHINE_BITS;
const TIMESTAMP_SHIFT: u64 = DATACENTER_SHIFT + DATACENTER_BITS;

/// Max values
const MAX_DATACENTER: u64 = (1 << DATACENTER_BITS) - 1;
const MAX_MACHINE: u64 = (1 << MACHINE_BITS) - 1;
const MAX_SEQUENCE: u64 = (1 << SEQUENCE_BITS) - 1;

/// Twitter custom epoch: Nov 04 2010 01:42:54 UTC
const CUSTOM_EPOCH: u64 = 1288834974657;

/// Snowflake ID generator
pub struct Snowflake {
    datacenter_id: u64,
    machine_id: u64,
    sequence: AtomicU64,
    last_timestamp: AtomicU64,
}

impl Snowflake {
    /// Create a new Snowflake generator
    pub fn new(datacenter_id: u64, machine_id: u64) -> Self {
        if datacenter_id > MAX_DATACENTER {
            panic!(
                "datacenter_id {} out of range (max {})",
                datacenter_id, MAX_DATACENTER
            );
        }
        if machine_id > MAX_MACHINE {
            panic!(
                "machine_id {} out of range (max {})",
                machine_id, MAX_MACHINE
            );
        }

        Snowflake {
            datacenter_id,
            machine_id,
            sequence: AtomicU64::new(0),
            last_timestamp: AtomicU64::new(0),
        }
    }

    /// Get current timestamp in milliseconds since epoch
    fn current_timestamp() -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System clock error");
        now.as_millis() as u64
    }

    /// Wait until the next millisecond
    fn wait_next_millis(last: u64) -> u64 {
        let mut ts = Snowflake::current_timestamp();
        while ts <= last {
            ts = Snowflake::current_timestamp();
        }
        ts
    }

    /// Generate the next unique 64-bit ID
    pub fn next_id(&self) -> u64 {
        let mut timestamp = Snowflake::current_timestamp();
        let last_ts = self.last_timestamp.load(Ordering::Relaxed);

        if timestamp < last_ts {
            // Clock rollback detected: wait until safe
            timestamp = Snowflake::wait_next_millis(last_ts);
        }

        let seq = if timestamp == last_ts {
            let next = (self.sequence.load(Ordering::Relaxed) + 1) & MAX_SEQUENCE;
            if next == 0 {
                // Sequence exhausted in this millisecond, wait for next
                timestamp = Snowflake::wait_next_millis(last_ts);
            }
            next
        } else {
            0
        };

        self.sequence.store(seq, Ordering::Relaxed);
        self.last_timestamp.store(timestamp, Ordering::Relaxed);

        ((timestamp - CUSTOM_EPOCH) << TIMESTAMP_SHIFT)
            | (self.datacenter_id << DATACENTER_SHIFT)
            | (self.machine_id << MACHINE_SHIFT)
            | seq
    }

    /// Decode an ID back into its components
    pub fn decode(id: u64) -> (u64, u64, u64, u64) {
        let sequence = id & MAX_SEQUENCE;
        let machine = (id >> MACHINE_SHIFT) & MAX_MACHINE;
        let datacenter = (id >> DATACENTER_SHIFT) & MAX_DATACENTER;
        let timestamp = (id >> TIMESTAMP_SHIFT) + CUSTOM_EPOCH;
        (timestamp, datacenter, machine, sequence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snowflake_id_generation() {
        let generator = Snowflake::new(1, 1);
        let id1 = generator.next_id();
        let id2 = generator.next_id();
        assert!(id2 > id1, "IDs should be monotonically increasing");
    }

    #[test]
    fn test_unique_and_ordered() {
        let gen = Snowflake::new(1, 1);
        let mut last = 0;
        for _ in 0..1000 {
            let id = gen.next_id();
            assert!(id > last, "IDs must be ordered");
            last = id;
        }
    }

    #[test]
    fn test_decode() {
        let gen = Snowflake::new(2, 3);
        let id = gen.next_id();
        let (ts, dc, mc, seq) = Snowflake::decode(id);

        assert_eq!(dc, 2);
        assert_eq!(mc, 3);
        assert!(seq >= 0);
        assert!(ts >= CUSTOM_EPOCH);
    }
}

fn main() {
    let gen = Snowflake::new(1, 1);

    for _ in 0..10 {
        let id = gen.next_id();
        let (ts, dc, mc, seq) = Snowflake::decode(id);
        println!(
            "id = {}, ts = {}, dc = {}, mc = {}, seq = {}",
            id, ts, dc, mc, seq
        );
    }
}
