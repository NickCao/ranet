use sha2::{Digest, Sha256};
use std::collections::HashMap;

// Zero disables XFRM binding; the top two values request %unique/%unique-dir.
const LAST_ID: u32 = u32::MAX - 2;

#[derive(Default)]
pub(crate) struct InterfaceIds {
    allocated: HashMap<u32, (String, String)>,
}

impl InterfaceIds {
    pub fn allocate(&mut self, local: &str, remote: &str) -> u32 {
        let mut hash = Sha256::new();
        hash.update(b"ranet-xfrm-if-id-v1\0");
        for identity in [local, remote] {
            hash.update((identity.len() as u64).to_be_bytes());
            hash.update(identity.as_bytes());
        }
        let hash = hash.finalize();
        let preferred = u32::from_be_bytes(hash[..4].try_into().unwrap());
        self.allocate_from(preferred, local, remote)
    }

    fn allocate_from(&mut self, preferred: u32, local: &str, remote: &str) -> u32 {
        let mut id = if (1..=LAST_ID).contains(&preferred) {
            preferred
        } else {
            1
        };
        loop {
            match self.allocated.get(&id) {
                Some((assigned_local, assigned_remote))
                    if assigned_local == local && assigned_remote == remote =>
                {
                    return id;
                }
                Some(_) => id = if id == LAST_ID { 1 } else { id + 1 },
                None => {
                    self.allocated
                        .insert(id, (local.to_owned(), remote.to_owned()));
                    return id;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stable_hash_vectors_and_identity_boundaries() {
        let mut ids = InterfaceIds::default();
        assert_eq!(ids.allocate("local", "remote"), 579_232_153);
        assert_eq!(ids.allocate("remote", "local"), 713_725_005);
        assert_eq!(ids.allocate("a", "bc"), 3_969_177_289);
        assert_eq!(ids.allocate("ab", "c"), 502_884_396);
    }

    #[test]
    fn hash_collision_uses_next_free_id_and_reuses_assignments() {
        // These identities have the same actual 32-bit SHA-256 prefix.
        let mut ids = InterfaceIds::default();
        assert_eq!(ids.allocate("local", "remote-45262"), 2_640_196_657);
        assert_eq!(ids.allocate("local", "remote-61022"), 2_640_196_658);
        assert_eq!(ids.allocate("local", "remote-45262"), 2_640_196_657);
        assert_eq!(ids.allocate("local", "remote-61022"), 2_640_196_658);
    }

    #[test]
    fn probing_wraps_and_skips_reserved_values() {
        let mut ids = InterfaceIds::default();
        assert_eq!(ids.allocate_from(LAST_ID, "local", "a"), LAST_ID);
        assert_eq!(ids.allocate_from(LAST_ID, "local", "b"), 1);
        assert_eq!(ids.allocate_from(0, "local", "c"), 2);
        assert_eq!(ids.allocate_from(u32::MAX - 1, "local", "d"), 3);
        assert_eq!(ids.allocate_from(u32::MAX, "local", "e"), 4);
    }
}
