use std::collections::{BTreeMap, HashMap};

use serde::{Serialize, Serializer};

/// See <https://stackoverflow.com/questions/42723065/how-to-sort-hashmap-keys-when-serializing-with-serde>
pub fn ordered_map<S, K, V>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    K: Serialize + std::cmp::Ord,
    V: Serialize,
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}
