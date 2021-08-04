
//! # fuzzy_rocks Overview
//! 
//! A persistent datastore backed by [RocksDB](https://rocksdb.org) with fuzzy key lookup using an arbitrary
//! distance function accelerated by the [SymSpell](https://github.com/wolfgarbe/SymSpell) algorithm.
//! 
//! The reasons to use this crate over another SymSpell implementation are:
//! - You have non-character-based keys (e.g. DNA snippets, etc.)
//! - You want to use a custom distance function
//! - Startup time matters more than lookups-per-second
//! - You have millions of keys and care about memory footprint
//! 
//! ## Records
//! 
//! This crate manages records, each of which has a unique [RecordID].  Keys are used to perform fuzzy
//! lookups but keys are not guaranteed to be unique. [Insert](Table::insert)ing the same key into a [Table]
//! twice will result in two distinct records, and both records will be found by lookups of that key.
//! 
//! ## Usage Example
//! 
//! ```
//! use fuzzy_rocks::{*};
//! 
//! //Create and reset the FuzzyRocks Table
//! let mut table = Table::new("test.rocks", DEFAULT_UTF8_TABLE).unwrap();
//! table.reset().unwrap();
//!
//! //Insert some records
//! let thu = table.insert("Thursday", &"Mokuyoubi".to_string()).unwrap();
//! let wed = table.insert("Wednesday", &"Suiyoubi".to_string()).unwrap();
//! let tue = table.insert("Tuesday", &"Kayoubi".to_string()).unwrap();
//! let mon = table.insert("Monday", &"Getsuyoubi".to_string()).unwrap();
//! 
//! //Try out lookup_best, to get the closest fuzzy match
//! let result = table.lookup_best("Bonday")
//!     .unwrap().next().unwrap();
//! assert_eq!(result, mon);
//! 
//! //Try out lookup_fuzzy, to get all matches and their distances
//! let results : Vec<(RecordID, u8)> = table
//!     .lookup_fuzzy("Tuesday", 2)
//!     .unwrap().collect();
//! assert_eq!(results.len(), 2);
//! assert!(results.contains(&(tue, 0))); //Tuesday -> Tuesday with 0 edits
//! assert!(results.contains(&(thu, 2))); //Thursday -> Tuesday with 2 edits
//! 
//! //Retrieve the key and value from a record
//! assert_eq!(table.get_one_key(wed).unwrap(), "Wednesday");
//! assert_eq!(table.get_value(wed).unwrap(), "Suiyoubi");
//! ```
//! 
//! Additional usage examples can be found in the tests, located at the bottom of the `src/lib.rs` file.
//! 
//! ## Distance Functions
//! 
//! A distance function is any function that returns a scalar distance between two keys.  The smaller the
//! distance, the closer the match.  Two identical keys must have a distance of [zero](num_traits::Zero).  The `fuzzy` methods
//! in this crate, such as [lookup_fuzzy](Table::lookup_fuzzy), require a distance function to be supplied.
//! 
//! This crate includes a simple [Levenstein Distance](https://en.wikipedia.org/wiki/Levenshtein_distance) function
//! called [edit_distance](Table::edit_distance).  However, you may often want to use a different function.
//! 
//! One reason to use a custom distance function is to account for expected variation patterns. For example:
//! a distance function that considers likely [OCR](https://en.wikipedia.org/wiki/Optical_character_recognition)
//! errors might consider 'lo' to be very close to 'b', '0' to be extremely close to 'O', and 'A' to be
//! somewhat near to '^', while '#' would be much further from ',' even though the Levenstein distances
//! tell a different story with 'lo' being two edits away from 'b' and '#' being only one edit away from
//! ',' (comma).
//! 
//! You may have a different distance function to catch common typos on a QWERTY keyboard, etc.
//! 
//! Another reason for a custom distance function is if your keys are not human-readable strings, in which
//! case you may need a different interpretation of variances between keys.  For example DNA snippets could
//! be used as keys.
//! 
//! Any distance function you choose must be compatible with SymSpell's delete-distance optimization.  In other
//! words, you must be able to delete no more than [config.max_deletes] characters from each of the record's
//! key and the lookup key and arrive at identical key-variants.  If your distance function is incompatible
//! with this property then the SymSpell optimization won't work for you and you should use a different fuzzy
//! lookup technique and a different crate.
//! 
//! Distance functions may return any scalar type, so floating point distances will work.  However, the
//! [config.max_deletes] constant is an integer.  Records that can't be reached by deleting `config.max_deletes` characters
//! from both the record key and the lookup key will never be evaluated by the distance function and are
//! conceptually "too far away".  Once the distance function has been evaluated, its return value is
//! considered the authoritative distance and the delete distance is irrelevant.
//! 
//! ## Unicode and UTF-8 Support
//! 
//! GOATGOAT, More to say
//! 
//! A [Table] may allow for unicode keys or not, depending on the value of the `UTF8_KEYS` constant used
//! when the Table was created.
//! 
//! If `UTF8_KEYS` is `true`, keys may use unicode characters and multi-byte characters will still be
//! considered as single characters for the purpose of deleting characters to create key variants.
//! 
//! If `UTF8_KEYS` is `false`, keys are just strings of [u8] characters.
//! This option has better performance.
//! 
//! ## Algorithm Details
//! 
//! The authoritative description of SymSpell is the ReadMe for the [SymSpell project](https://github.com/wolfgarbe/SymSpell).
//! 
//! The fuzzy_rocks implementation has a few additional details to be aware of:
//! 
//! - fuzzy_rocks won't find keys that don't have at least one character in common, regardless of the value
//! of `config.max_deletes`.  For example the key `me` won't be found by the query string `hi`, even with a distance
//! of 2 (or any other value).  This decision was made because the variant space becomes very crowded
//! for short keys, and the extreme example of the empty-string variant was severely damaging performance with
//! short keys.
//! 
//! ## Performance Characteristics
//! 
//! This crate is designed for large databases where startup time and resident memory footprint are significant
//! considerations.  This create has been tested with 200,000 records cumulatively having over 1 million keys,
//! and about 140 million key variants.  In this situation, a fuzzy lookup was about 500 microseconds running
//! on my laptop - which is very expensive in absolute terms.
//! 
//! The performance will also vary greatly depending on the key distribution and the table parameters.  Keys
//! that are distinct from eachother will lead to faster searches vs. keys that share many variants in common.
//! 
//! GOATGOAT, explain multi-key support
//! 
//! ### Tuning for Performance
//! 
//! GOATGOATGOAT, Write-up on perf_counters, how to enable, etc.
//! 
//! GOATGOATGOAT, Write-up on benchmarks.  How to edit, how to run.
//! 
//! A smaller `config.max_deletes` value will perform better but be able to find fewer results for a search.
//! 
//! A higher value for `config.meaningful_key_len` will result in fewer wasted evaluations of the distance function
//! but will lead to more entries in the variants database and thus more memory pressure.
//! 
//! If your use-case can cope with a higher startup latency and you are ok with all of your keys and
//! variants being loaded into memory, then query performance will certainly be better using a solution
//! built on Rust's native collections, such as this [symspell](https://crates.io/crates/symspell)
//! crate on [crates.io](http://crates.io).
//! 
//! ## Misc
//! 
//! **NOTE**: The included `geonames_megacities.txt` file is a stub for the `geonames_test`, designed to stress-test
//! this crate.  The abriged file is included so the test will pass regardless, and to avoid bloating the
//! download.  The content of `geonames_megacities.txt` was derived from data on [geonames.org](http://geonames.org),
//! and licensed under a [Creative Commons Attribution 4.0 License](https://creativecommons.org/licenses/by/4.0/legalcode)
//! 

//GOATGOATGOAT, write up:
// 0.) DNA snippet example.  Look at FAStA Wikipedia article
// 1.) How SymSpell works best with sparse key spaces, but a BK-Tree is better for dense key spaces
// 2.) How we have "Future Work" to add a BK tree to search within a key group
//      *consider adding a "k-nn query", in addition to or instead of the "best" query
//      *Tools to detect when a parameter is tuned badly for a data set, based on known optimal
//      ratios for certain performance counters.  Alternatively, tools to assist in performing a
//      config-parameter optimization process, to tune a table config to a data set.
//      *Save the Table config to the database (and the checksum of the distance function) to
//      detect an error when the config changes in a way that makes the database invalid

//GOATGOATGOAT
//Create a parameter block for generic initialization.
// The distance function should be part of the parameter block, and therefore part of the table

use core::marker::PhantomData;
use core::cmp::{min, Ordering};
use core::hash::Hash;

#[cfg(feature = "perf_counters")]
use core::cell::Cell;

use num_traits::Zero;

use serde::{Serialize, Deserialize};
use bincode::Options;

use std::{slice};
use std::collections::{HashMap, HashSet};
use std::convert::{TryInto};
use std::iter::FromIterator;
use std::mem::{MaybeUninit, forget, size_of, transmute};

use rocksdb::{DB, DBWithThreadMode, ColumnFamily, ColumnFamilyDescriptor, MergeOperands};

mod unicode_string_helpers;
use unicode_string_helpers::{*};

mod bincode_helpers;
use bincode_helpers::{*};

/// A collection containing records that may be searched by `key`
/// 
/// 
/// 
/// -`UTF8_KEYS` specifies whether the keys are UTF-8 encoded strings or not.  UFT-8 awareness is
/// required to avoid deleting partial characters thus rendering the string invalid.  This comes at a
/// performance cost, however, so passing `false` is more efficient if you plan to use regular ascii or
/// any other kind of data as the table's keys.
/// 
pub struct Table<KeyCharT, DistanceT, ValueT, const UTF8_KEYS : bool> {
    record_count : usize,
    db : DBWithThreadMode<rocksdb::SingleThreaded>,
    config : TableConfig<KeyCharT, DistanceT, ValueT, UTF8_KEYS>,
    path : String,
    deleted_records : Vec<RecordID>, //NOTE: Currently we don't try to hold onto deleted records across unloads, but we may change this in the future.
    #[cfg(feature = "perf_counters")]
    perf_counters : Cell<PerfCounters>,
    phantom_key: PhantomData<KeyCharT>,
    phantom_value: PhantomData<ValueT>,
}

// NOTE: The #![feature(const_generics)] feature isn't stabilized and I don't want to depend on
// any unstable features.  So instead of taking the config structure as a const parameter to
// Table, the compile-time arguments will be passed individually using the capabilities of
// #![feature(min_const_generics)], and I'll hide that from novice API users by allowing the
// compiler to infer the values from phantoms in the config structure.

#[derive(Clone)]
pub struct TableConfig<KeyCharT, DistanceT, ValueT, const UTF8_KEYS : bool> {

    /// The number of deletes to store in the database for variants created
    /// by the SymSpell optimization.  If `max_deletes` is too small, the variant will not be found
    /// and therefore the `distance_function` will not have an opportunity to evaluate the match.  However,
    /// if `max_deletes` is too large, it will hurt performance by evaluating too many candidates.
    /// 
    /// Empirically, values near 2 seem to be good in most situations I have found.  I.e. 1 and 3 might be
    /// appropriate sometimes.  4 ends up exploding in most cases I've seen so the SymSpell logic may not
    /// be a good fit if you need to find keys 4 edits away.  0 edits is an exact match.
    pub max_deletes : usize,

    /// This controls an optimization where only a subset of the key is used for creating
    /// variants.  For example, if `meaningful_key_len = 10` then only the first 10 characters of the key will be used
    /// to generate and search for variants.
    /// 
    /// This optimization is predicated on the idea that long key strings will not be very similar to each
    /// other, and a certain number of characters is sufficient to substantially narrow down the search.
    /// 
    /// For example the key *incomprehensibilities* will cause variants to be generated for *incomprehe*
    /// with a `meaningful_key_len` of 10, meaning that a search for *incomprehension* would find *incomprehensibilities*
    /// and evauate it with the `distance_function` even though it is further than [config.max_deletes].
    /// 
    /// In a dataset where many keys share a common prefix, or where keys are organized into a namespace by
    /// concatenating strings, this optimization will cause problems and you should either pass a high number
    /// to effectively disable it, or rework this code to use different logic to select the substring
    /// 
    /// Lookup functions that invoke the distance function will always use the entire key, regardless of the
    /// value of `meaningful_key_len`.
    pub meaningful_key_len : usize,

    /// The number of variants a key must share with a keys in an existing key group in order for that key to
    /// be added to the group rather than being placed into a new separate key group.
    /// 
    /// NOTE: We arrived at the default value (5) empirically by testing a number of different values and
    /// observing the effects on lookup speed, DB construction speed, and DB size.  The observed data
    /// points are checked in, in the file: `misc/perf_data.txt`
    pub group_variant_overlap_threshold : usize,
    pub distance_function : fn(key_a : &[KeyCharT], key_b : &[KeyCharT]) -> DistanceT,
    phantom_key: PhantomData<KeyCharT>,
    phantom_distance: PhantomData<DistanceT>,
    phantom_value: PhantomData<ValueT>,
}

impl <KeyCharT : 'static + Copy + Eq + Hash + Serialize + serde::de::DeserializeOwned, DistanceT : 'static + Copy + Zero + PartialOrd + PartialEq + From<u8>, ValueT, const UTF8_KEYS : bool>TableConfig<KeyCharT, DistanceT, ValueT, UTF8_KEYS> {

    pub fn default() -> Self {
        Self {
            max_deletes : 2,
            meaningful_key_len : 12,
            group_variant_overlap_threshold : 5,
            distance_function : Self::edit_distance,
            phantom_key : PhantomData,
            phantom_distance : PhantomData,
            phantom_value : PhantomData,        
        }
    }

    /// An implementation of the basic Levenstein distance function, which may be passed to
    /// [lookup_fuzzy](Table::lookup_fuzzy), [lookup_best](Table::lookup_best), or used anywhere
    /// else a distance function is needed.
    /// 
    /// This implementation uses the Wagner-Fischer Algorithm, as it's described [here](https://en.wikipedia.org/wiki/Levenshtein_distance)
    pub fn edit_distance(key_a : &[KeyCharT], key_b : &[KeyCharT]) -> DistanceT {

        let m = key_a.len()+1;
        let n = key_b.len()+1;

        //Allocate a 2-dimensional vec for the distances between the first i characters of key_a
        //and the first j characters of key_b
        let mut d : [[u8; MAX_KEY_LENGTH + 1]; MAX_KEY_LENGTH + 1] = unsafe { MaybeUninit::uninit().assume_init() };

        //NOTE: I personally find this (below) more readable, but clippy really like the other style.  -\_(..)_/-
        // for i in 1..m {
        //     d[i][0] = i;
        // }
        for (i, row) in d.iter_mut().enumerate().skip(1) {
            //row[0] = i as u8;
            let element = unsafe{ row.get_unchecked_mut(0) };
            *element = i as u8;
        }

        // for j in 1..n {
        //     d[0][j] = j as u8;
        // }
        for (j, element) in d[0].iter_mut().enumerate() {
            *element = j as u8;
        }

        for j in 1..n {
            for i in 1..m {

                //TODO: There is one potential optimization left.  There is no reason to allcate a whole
                // square buffer of MAX_KEY_LENGTH on a side, because we never look back before the previous
                // line in the buffer.  A more cache-friendly approach might be to allocate a
                // MAX_KEY_LENGTH x 2 buffer, and then alternate between writing the values in one line
                // and reading them from the previous line.

                let substitution_cost = if key_a[i-1] == key_b[j-1] {
                    0
                } else {
                    1
                };

                //let deletion_distance = d[i-1][j] + 1;
                let deletion_distance = unsafe {d.get_unchecked(i-1).get_unchecked(j)} + 1;
                //let insertion_distance = d[i][j-1] + 1;
                let insertion_distance = unsafe {d.get_unchecked(i).get_unchecked(j-1)} + 1;
                //let substitution_distance = d[i-1][j-1] + substitution_cost;
                let substitution_distance = unsafe {d.get_unchecked(i-1).get_unchecked(j-1)} + substitution_cost;

                let smallest_distance = min(min(deletion_distance, insertion_distance), substitution_distance);
                
                //d[i][j] = smallest_distance;  
                let element = unsafe{ d.get_unchecked_mut(i).get_unchecked_mut(j) };
                *element = smallest_distance;
            }
        }

        DistanceT::from(d[m-1][n-1])
    }
}

pub const DEFAULT_UTF8_TABLE : TableConfig<char, u8, String, true> = TableConfig {
    max_deletes : 2,
    meaningful_key_len : 12,
    group_variant_overlap_threshold : 5,
    distance_function : TableConfig::<char, u8, String, true>::edit_distance,
    phantom_key : PhantomData,
    phantom_distance : PhantomData,
    phantom_value : PhantomData,
};

/// Performance counters for optimizing [Table] parameters.
/// 
/// These counters don't reflect stats and totals across the whole database, rather they can
/// be reset and therefore used to measure individual operations or sequences of operations.
/// 
/// NOTE: Counters are being implemented on an as-needed basis, and many are still unimplimented
#[cfg(feature = "perf_counters")]
#[derive(Debug, Copy, Clone)]
struct PerfCounters {

    variant_load_count : usize, //The number of times we load a variant entry from the DB during a fuzzy lookup
    key_group_lookup_count : usize, //The number of time we load the keys from a key_group
    keys_found_count : usize, //The number of keys we find across all records we lookup the keys for
    max_variant_entry_refs : usize, // The number of RecordIDs in the variant with the most RecordIDs, among all variants loaded during lookups
}

#[cfg(feature = "perf_counters")]
impl PerfCounters {
    pub fn new() -> Self {
        Self {
            variant_load_count : 0,
            key_group_lookup_count : 0,
            keys_found_count : 0,
            max_variant_entry_refs : 0,
        }
    }
}

/// A unique identifier for a record within a [Table]
/// 
/// NOTE: although the RecordID is 64 bits, only 44 bits can be used to address
/// records, giving a theoretical maximum of 17.6 trillion records although
/// this crate has only been tested with about 10 million unique records, so far
#[derive(Copy, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, derive_more::Display, Serialize, Deserialize)]
pub struct RecordID(usize);
impl RecordID {
    pub const NULL : RecordID = RecordID(usize::MAX);
}

/// A unique identifier for a key group, which includes its RecordID
/// 
/// Lower 44 bits are the RecordID, upper 20 bits are the GroupID
#[derive(Copy, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, derive_more::Display, Serialize, Deserialize)]
struct KeyGroupID(usize);
impl KeyGroupID {
    fn from_record_and_idx(record_id : RecordID, group_idx : usize) -> Self {
        //Panic if we have more than the allowed number of records
        if record_id.0 > 0xFFFFFFFFFFF {
            panic!("too many records!");
        }
        let record_component = record_id.0 & 0xFFFFFFFFFFF;
        let group_component = group_idx << 44;
        Self(record_component + group_component)
    }
    fn record_id(&self) -> RecordID {
        RecordID(self.0 & 0xFFFFFFFFFFF)
    }
    fn group_idx(&self) -> usize {
        self.0 >> 44
    }
}

/// Some meta-data associated with each record
#[derive(Serialize, Deserialize)]
struct RecordData {
    key_groups : Vec<usize>,
    //DANGER: If any additional fields are added here, we must update `put_key_groups` to preserve
    // other fields before just overwriting it.
}

impl RecordData {
    fn new(key_groups : &[usize]) -> Self {
        Self{
            key_groups : key_groups.to_vec()
        }
    }
}

/// A transient struct to keep track of the multiple key groups, reflecting what's in the DB.
/// Used when assembling key groups or adding new keys to existing groups
#[derive(Clone)]
struct KeyGroups<OwnedKeyT> {
    variant_reverse_lookup_map : HashMap<Vec<u8>, usize>,
    key_group_variants : Vec<HashSet<Vec<u8>>>,
    key_group_keys : Vec<HashSet<OwnedKeyT>>,
    group_ids : Vec<usize> //The contents of this vec correspond to the KeyGroupID
}

impl <OwnedKeyT>KeyGroups<OwnedKeyT> {
    fn new() -> Self {
        Self{
            variant_reverse_lookup_map : HashMap::new(),
            key_group_variants : vec![],
            key_group_keys : vec![],
            group_ids : vec![]
        }
    }
    fn next_available_group_id(&self) -> usize {
        //It doesn't matter if we leave some holes, but we must not collide, therefore we'll
        //start at the length of the vec, and search forward from there
        let mut group_id = self.group_ids.len();
        while self.group_ids.contains(&group_id) {
            group_id += 1;
        }
        group_id
    }
}

/// The maximum number of characters allowable in a key.  Longer keys will cause an error
pub const MAX_KEY_LENGTH : usize = 95;

const KEYS_CF_NAME : &str = "keys";
const RECORD_DATA_CF_NAME : &str = "rec_data";
const VALUES_CF_NAME : &str = "values";
const VARIANTS_CF_NAME : &str = "variants";

/// A private trait representing the subset of key types that are owned and therefore 'static
//GOAT, make this trait private, which means refactoring the modules
pub trait OwnedKey<KeyCharT> : 'static + Sized + Serialize + serde::de::DeserializeOwned + Key<'static, KeyCharT> {
    fn as_string(&self) -> Option<String>;
    fn borrow_str(&self) -> Option<&str>;
    fn as_vec(&self) -> Option<Vec<KeyCharT>>;
    fn borrow_vec(&self) -> Option<&[KeyCharT]>;
}

impl OwnedKey<char> for String {
    fn as_string(&self) -> Option<String> {
        Some(self.clone())
    }
    fn borrow_str(&self) -> Option<&str> {
        Some(self)
    }
    fn as_vec(&self) -> Option<Vec<char>> {
        Some(self.chars().collect())
    }
    fn borrow_vec(&self) -> Option<&[char]> {
        None
    }
}

impl <KeyCharT : 'static + Copy + Eq + Hash + Serialize + serde::de::DeserializeOwned>OwnedKey<KeyCharT> for Vec<KeyCharT> {
    fn as_string(&self) -> Option<String> {
        None
    }
    fn borrow_str(&self) -> Option<&str> {
        None
    }
    fn as_vec(&self) -> Option<Vec<KeyCharT>> {
        Some(self.clone())
    }
    fn borrow_vec(&self) -> Option<&[KeyCharT]> {
        Some(&self[..])
    }
}

/// A private trait implemented by a [Table] to provide access to the keys in the DB, 
/// whether they are UTF-8 encoded strings or arrays of KeyCharT
//GOAT, Make this private by moving it to a private module
pub trait TableKeyEncoding<KeyCharT> {
    type OwnedKeyT : OwnedKey<KeyCharT>;

    fn owned_key_into_vec(key : Self::OwnedKeyT) -> Vec<KeyCharT>;
    fn owned_key_into_buf<'a>(key : &'a Self::OwnedKeyT, buf : &'a mut Vec<KeyCharT>) -> &'a Vec<KeyCharT>;
    fn owned_key_from_string(s : String) -> Self::OwnedKeyT;
    fn owned_key_from_vec(v : Vec<KeyCharT>) -> Self::OwnedKeyT;
    fn owned_key_from_key<'a, K : Key<'a, KeyCharT>>(k : &K) -> Self::OwnedKeyT;
}

impl <DistanceT, ValueT>TableKeyEncoding<char> for Table<char, DistanceT, ValueT, true> {
    type OwnedKeyT = String;
    
    fn owned_key_into_vec(key : Self::OwnedKeyT) -> Vec<char> {
        //NOTE: 15% of the performance on the fuzzy lookups was taken with this function before I
        // started optimizing and utimately switched over to owned_key_into_buf. It appeared that
        // the buffer was being reallocated for each additional char for the collect() implementation.

        //NOTE: Old implementation
        // key.chars().collect()

        //NOTE: It appears that this implementation is twice as fast as the collect() implementation,
        // but we're still losing 7% of overall perf allocating and freeing this Vec, so I'm switching
        // to owned_key_into_buf().
        let num_chars = key.num_chars();
        let mut result_vec = Vec::with_capacity(num_chars);
        for the_char in key.chars() {
            result_vec.push(the_char);
        }
        result_vec
    }

    fn owned_key_into_buf<'a>(key : &'a Self::OwnedKeyT, buf : &'a mut Vec<char>) -> &'a Vec<char> {

        let mut num_chars = 0;
        for (i, the_char) in key.chars().enumerate() {
            let element = unsafe{ buf.get_unchecked_mut(i) };
            *element = the_char;
            num_chars = i;
        }
        unsafe{ buf.set_len(num_chars+1) };

        buf
    }

    fn owned_key_from_string(s : String) -> Self::OwnedKeyT {
        s
    }
    fn owned_key_from_vec(_v : Vec<char>) -> Self::OwnedKeyT {
        panic!() //NOTE: Should never be called when the OwnedKeyT isn't a Vec
    }
    fn owned_key_from_key<'a, K : Key<'a, char>>(k : &K) -> Self::OwnedKeyT {
        k.borrow_key_str().unwrap().to_string() //NOTE: the unwrap() will panic if called with the wrong kind of key
    }
}
impl <KeyCharT : 'static + Copy + Eq + Hash + Serialize + serde::de::DeserializeOwned, DistanceT, ValueT>TableKeyEncoding<KeyCharT> for Table<KeyCharT, DistanceT, ValueT, false> {
    type OwnedKeyT = Vec<KeyCharT>;
    
    fn owned_key_into_vec(key : Self::OwnedKeyT) -> Vec<KeyCharT> {
        key
    }

    fn owned_key_into_buf<'a>(key : &'a Self::OwnedKeyT, _buf : &'a mut Vec<KeyCharT>) -> &'a Vec<KeyCharT> {
        key
    }

    fn owned_key_from_string(_s : String) -> Self::OwnedKeyT {
        panic!() //NOTE: Should never be called when the OwnedKeyT isn't a String
    }
    fn owned_key_from_vec(v : Vec<KeyCharT>) -> Self::OwnedKeyT {
        v
    }
    fn owned_key_from_key<'a, K : Key<'a, KeyCharT>>(k : &K) -> Self::OwnedKeyT {
        k.get_key_chars()
    }
}

/// Implemented by all types that can be used as keys, whether they are UTF-8 encoded
/// strings or arrays of KeyCharT
pub trait Key<'a, KeyCharT> : KeyUnsafe<'a, KeyCharT> {

    fn num_chars(&self) -> usize;
    fn as_bytes(&self) -> &[u8];
    fn into_bytes(self) -> Vec<u8>;
    fn borrow_key_chars(&self) -> Option<&[KeyCharT]>;
    fn get_key_chars(&self) -> Vec<KeyCharT>;
    fn borrow_key_str(&self) -> Option<&str>;
    fn from_owned<OwnedKeyT : OwnedKey<KeyCharT>>(owned_key : &'a OwnedKeyT) -> Self;
}

/// The private unsafe accessors for the Key trait
//GOATGOAT, Make this private, which means refactoring the modules.
pub trait KeyUnsafe<'a, KeyCharT> : Eq + Hash + Clone + Serialize {
    /// This function may return a result that borrows the owned_key parameter, but the
    /// returned result may have a longer lifetime on account of the type it's called with
    unsafe fn from_owned_unsafe<OwnedKeyT : OwnedKey<KeyCharT>>(owned_key : &OwnedKeyT) -> Self;
}

impl <'a, KeyCharT>Key<'a, KeyCharT> for &'a [KeyCharT]
    where
    KeyCharT : 'static + Copy + Eq + Hash + Serialize + serde::de::DeserializeOwned,
{
    fn num_chars(&self) -> usize {
        self.len()
    }

    fn as_bytes(&self) -> &[u8] {
        let len = self.len();
        unsafe { slice::from_raw_parts(self.as_ptr() as *const u8, size_of::<KeyCharT>() * len) }
    }

    fn into_bytes(self) -> Vec<u8> {

        let mut owned_vec = self.to_vec();
        let len = owned_vec.len();
        let cap = owned_vec.capacity();

        //Now transmute the vec into a vec of bytes
        let result = unsafe { Vec::<u8>::from_raw_parts(owned_vec.as_mut_ptr() as *mut u8, size_of::<KeyCharT>() * len, size_of::<KeyCharT>() * cap) };
        forget(owned_vec); //So we don't get a double-free
        result
    }

    fn borrow_key_chars(&self) -> Option<&[KeyCharT]> {
        Some(self)
    }

    fn get_key_chars(&self) -> Vec<KeyCharT> {
        self.to_vec()
    }

    fn borrow_key_str(&self) -> Option<&str> {
        None
    }

    fn from_owned<OwnedKeyT : OwnedKey<KeyCharT>>(owned_key : &'a OwnedKeyT) -> Self {
        owned_key.borrow_vec().unwrap()
    }
}

impl <'a, KeyCharT>KeyUnsafe<'a, KeyCharT> for &'a [KeyCharT]
    where
    KeyCharT : 'static + Copy + Eq + Hash + Serialize + serde::de::DeserializeOwned,
{
    unsafe fn from_owned_unsafe<OwnedKeyT : OwnedKey<KeyCharT>>(owned_key : &OwnedKeyT) -> Self {
        let result = owned_key.borrow_vec().unwrap();
        transmute::<&[KeyCharT], &'a [KeyCharT]>(result)
    }
}

impl <KeyCharT>Key<'_, KeyCharT> for Vec<KeyCharT>
    where
    KeyCharT : 'static + Copy + Eq + Hash + Serialize + serde::de::DeserializeOwned,
{
    fn num_chars(&self) -> usize {
        self.len()
    }

    fn as_bytes(&self) -> &[u8] {
        let len = self.len();
        unsafe { slice::from_raw_parts(self.as_ptr() as *const u8, size_of::<KeyCharT>() * len) }
    }

    fn into_bytes(self) -> Vec<u8> {
        let mut mut_self = self;
        let len = mut_self.len();
        let cap = mut_self.capacity();

        let result = unsafe { Vec::<u8>::from_raw_parts(mut_self.as_mut_ptr() as *mut u8, size_of::<KeyCharT>() * len, size_of::<KeyCharT>() * cap) };
        forget(mut_self); //So we don't get a double-free

        result
    }

    fn borrow_key_chars(&self) -> Option<&[KeyCharT]> {
        Some(&self[..])
    }

    fn get_key_chars(&self) -> Vec<KeyCharT> {
        self.clone()
    }

    fn borrow_key_str(&self) -> Option<&str> {
        None
    }

    fn from_owned<OwnedKeyT : OwnedKey<KeyCharT>>(owned_key : &OwnedKeyT) -> Self {
        owned_key.as_vec().unwrap()
    }
}

impl <KeyCharT>KeyUnsafe<'_, KeyCharT> for Vec<KeyCharT>
    where
    KeyCharT : 'static + Copy + Eq + Hash + Serialize + serde::de::DeserializeOwned,
{
    unsafe fn from_owned_unsafe<OwnedKeyT : OwnedKey<KeyCharT>>(owned_key : &OwnedKeyT) -> Self {
        //This implementation is actually safe, but the fn prototype is unsafe
        owned_key.as_vec().unwrap()
    }
}

impl <'a>Key<'a, char> for &'a str
{
    fn num_chars(&self) -> usize {
        unicode_len(self)
    }

    fn as_bytes(&self) -> &[u8] {
        str::as_bytes(self)
    }

    fn into_bytes(self) -> Vec<u8> {
        let bytes_slice = str::as_bytes(self);
        bytes_slice.to_vec()
    }

    fn borrow_key_chars(&self) -> Option<&[char]> {
        None
    }

    fn get_key_chars(&self) -> Vec<char> {
        self.chars().collect()
    }

    fn borrow_key_str(&self) -> Option<&str> {
        Some(self)
    }

    fn from_owned<OwnedKeyT : OwnedKey<char>>(owned_key : &'a OwnedKeyT) -> Self {
        owned_key.borrow_str().unwrap()
    }

}

impl <'a>KeyUnsafe<'a, char> for &'a str {

    unsafe fn from_owned_unsafe<OwnedKeyT : OwnedKey<char>>(owned_key : &OwnedKeyT) -> Self {
        let result = owned_key.borrow_str().unwrap();
        transmute::<&str, &'a str>(result)
    }
}

impl Key<'_, char> for String
{
    fn num_chars(&self) -> usize {
        unicode_len(self)
    }

    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    fn into_bytes(self) -> Vec<u8> {
        self.into_bytes()
    }

    fn borrow_key_chars(&self) -> Option<&[char]> {
        None
    }

    fn get_key_chars(&self) -> Vec<char> {
        self.chars().collect()
    }

    fn borrow_key_str(&self) -> Option<&str> {
        Some(&self)
    }

    fn from_owned<OwnedKeyT : OwnedKey<char>>(owned_key : &OwnedKeyT) -> Self {
        owned_key.as_string().unwrap()
    }
}

impl KeyUnsafe<'_, char> for String {

    unsafe fn from_owned_unsafe<OwnedKeyT : OwnedKey<char>>(owned_key : &OwnedKeyT) -> Self {
        //This implementation is actually safe, but the fn prototype is unsafe
        owned_key.as_string().unwrap()
    }
}

/// The implementation of the shared parts of Table
impl <KeyCharT : 'static + Copy + PartialEq + Serialize + serde::de::DeserializeOwned, DistanceT : 'static + Copy + Zero + PartialOrd + PartialEq + From<u8>, ValueT : 'static + Serialize + serde::de::DeserializeOwned, const UTF8_KEYS : bool>Table<KeyCharT, DistanceT, ValueT, UTF8_KEYS>
    where Self : TableKeyEncoding<KeyCharT> {

    /// Creates a new Table, backed by the database at the path provided
    /// 
    /// WARNING:  No sanity checks are performed to ensure the database being opened matches the parameters
    /// of the table being created.  Therefore you may see bugs if you are opening a table that was created
    /// using a different set of parameters.
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn new(path : &str, config : TableConfig<KeyCharT, DistanceT, ValueT, UTF8_KEYS>) -> Result<Self, String> {

        //Configure the "keys" and "values" column families
        let keys_cf = ColumnFamilyDescriptor::new(KEYS_CF_NAME, rocksdb::Options::default());
        let rec_data_cf = ColumnFamilyDescriptor::new(RECORD_DATA_CF_NAME, rocksdb::Options::default());
        let values_cf = ColumnFamilyDescriptor::new(VALUES_CF_NAME, rocksdb::Options::default());

        //Configure the "variants" column family
        let mut variants_opts = rocksdb::Options::default();
        variants_opts.create_if_missing(true);
        variants_opts.set_merge_operator_associative("append to RecordID vec", Self::variant_append_merge);
        let variants_cf = ColumnFamilyDescriptor::new(VARIANTS_CF_NAME, variants_opts);

        //Configure the database itself
        let mut db_opts = rocksdb::Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);

        //Open the database
        let db = DB::open_cf_descriptors(&db_opts, path, vec![keys_cf, rec_data_cf, values_cf, variants_cf])?;

        //Find the maximum RecordID, by probing the entries in the "rec_data" column family
        let rec_data_cf_handle = db.cf_handle(RECORD_DATA_CF_NAME).unwrap();
        let record_count = probe_for_max_sequential_key(&db, rec_data_cf_handle, 255)?;

        Ok(Self {
            record_count,
            config : config,
            db,
            path : path.to_string(),
            deleted_records : vec![],
            #[cfg(feature = "perf_counters")]
            perf_counters : Cell::new(PerfCounters::new()),
            phantom_key : PhantomData,
            phantom_value : PhantomData
        })
    }

    /// Resets a Table, dropping every record in the table and restoring it to an empty state.
    /// 
    /// (Dropping in a database sense, not a Rust sense)
    pub fn reset(&mut self) -> Result<(), String> {
        
        //Drop all the existing column families
        self.db.drop_cf(KEYS_CF_NAME)?;
        self.db.drop_cf(RECORD_DATA_CF_NAME)?;
        self.db.drop_cf(VALUES_CF_NAME)?;
        self.db.drop_cf(VARIANTS_CF_NAME)?;

        //Recreate the "keys", "rec_data", and "values" column families
        self.db.create_cf(KEYS_CF_NAME, &rocksdb::Options::default())?;
        self.db.create_cf(RECORD_DATA_CF_NAME, &rocksdb::Options::default())?;
        self.db.create_cf(VALUES_CF_NAME, &rocksdb::Options::default())?;
        
        //Recreate the "variants" column family
        let mut variants_opts = rocksdb::Options::default();
        variants_opts.create_if_missing(true);
        variants_opts.set_merge_operator_associative("append to RecordID vec", Self::variant_append_merge);
        self.db.create_cf(VARIANTS_CF_NAME, &variants_opts)?;

        //Reset the record_count, so newly inserted entries begin at 0 again
        self.record_count = 0;
        Ok(())
    }

    /// Deletes a record from the Table.
    /// 
    /// A deleted record cannot be accessed or otherwise found, but the RecordID may be reassigned
    /// using [Table::replace].
    pub fn delete(&mut self, record_id : RecordID) -> Result<(), String> {

        self.delete_keys_internal(record_id)?;
        self.delete_value_internal(record_id)?;
        self.deleted_records.push(record_id);

        Ok(())
    }

    /// Deletes all of the keys belonging to a record, and all associated variants
    /// 
    /// Leaves the record in a half-composed state, so should only be called as part of another
    /// operation.
    fn delete_keys_internal(&mut self, record_id : RecordID) -> Result<(), String> {

        //Get all of the key-groups belonging to the record
        for key_group in self.get_key_groups(record_id)? {

            //Get all the keys for the group we're removing, so we can compute all the variants
            let keys_iter = self.get_keys_in_group(key_group)?;
            let mut variants = HashSet::new();
            for key in keys_iter {
                let key_variants = self.variants(&key);
                variants.extend(key_variants);
            }

            //Remove the variants' reference to this key group
            self.delete_variants_internal(key_group, variants)?;
            
            //Delete the key group entry in the table
            self.delete_key_group_entry(key_group)?;
        }

        //Now replace the key groups vec in the "rec_data" table with an empty sentinel vec
        //NOTE: We replace the record rather than delete it because we assume there are no gaps in the
        // RecordIDs, when assigning new a RecordID
        self.put_key_groups(record_id, &[])?;

        Ok(())
    }

    /// Deletes a key group entry from the db.  Does not clean up variants that may reference
    /// the key group, so must be called as part of another operation
    fn delete_key_group_entry(&mut self, key_group : KeyGroupID) -> Result<(), String> {
        
        let keys_cf_handle = self.db.cf_handle(KEYS_CF_NAME).unwrap();
        self.db.delete_cf(keys_cf_handle, key_group.0.to_le_bytes())?;

        Ok(())
    }

    fn delete_variants_internal(&mut self, key_group : KeyGroupID, variants : HashSet<Vec<u8>>) -> Result<(), String> {
        
        //Loop over each variant, and remove the KeyGroupID from its associated variant entry in
        // the database, and remove the variant entry if it only referenced the key_group we're removing
        let variants_cf_handle = self.db.cf_handle(VARIANTS_CF_NAME).unwrap();
        for variant in variants.iter() {

            if let Some(variant_entry_bytes) = self.db.get_pinned_cf(variants_cf_handle, variant)? {

                let variant_entry_len = bincode_vec_fixint_len(&variant_entry_bytes);

                //If the variant entry references more than one record, rebuild it with our records absent
                if variant_entry_len > 1 {
                    let mut new_vec : Vec<KeyGroupID> = Vec::with_capacity(variant_entry_len-1);
                    for key_group_id_bytes in bincode_vec_iter::<KeyGroupID>(&variant_entry_bytes) {
                        let other_key_group_id = KeyGroupID(usize::from_le_bytes(key_group_id_bytes.try_into().unwrap()));
                        if other_key_group_id != key_group {
                            new_vec.push(other_key_group_id);
                        }
                    }
                    let vec_coder = bincode::DefaultOptions::new().with_fixint_encoding().with_little_endian();
                    self.db.put_cf(variants_cf_handle, variant, vec_coder.serialize(&new_vec).unwrap())?;
                } else {
                    //Otherwise, remove the variant entry entirely
                    self.db.delete_cf(variants_cf_handle, variant)?;
                }
            }
        }

        Ok(())
    }

    /// Creates entries in the keys table.  If we are updating an old record, we will overwrite it.
    /// 
    /// NOTE: This function will NOT update any variants used to locate the key
    fn put_key_group_internal<K : Eq + Hash + Serialize>(&mut self, key_group_id : KeyGroupID, raw_keys : &HashSet<K>) -> Result<(), String> {
        
        //Serialize the keys into a vec of bytes
        let record_coder = bincode::DefaultOptions::new().with_varint_encoding().with_little_endian();
        let keys_bytes = record_coder.serialize(&raw_keys).unwrap();

        //Put the vector of keys into the keys table
        let keys_cf_handle = self.db.cf_handle(KEYS_CF_NAME).unwrap();
        self.db.put_cf(keys_cf_handle, usize::to_le_bytes(key_group_id.0), keys_bytes)?;

        Ok(())
    }

    /// Divides the keys up into key groups and assigns them to a record.
    /// 
    /// Should NEVER be called on a record that already has keys or orphaned database entries will result
    fn put_record_keys<'a, K : Key<'a, KeyCharT> + 'a, KeysIterT : Iterator<Item=&'a K>>(&mut self, record_id : RecordID, keys_iter : KeysIterT, num_keys : usize) -> Result<(), String> {
    
        //Make groups for the keys
        let groups = self.make_groups_from_keys(keys_iter, num_keys).unwrap();
        let num_groups = groups.key_group_keys.len();

        //Put the variants for each group into the right table
        for (idx, variant_set) in groups.key_group_variants.into_iter().enumerate() {
            let key_group_id = KeyGroupID::from_record_and_idx(record_id, idx); 
            self.put_variants_internal(key_group_id, variant_set)?;
        }
        
        //Put the keys for each group into the table
        for (idx, key_set) in groups.key_group_keys.into_iter().enumerate() {
            let key_group_id = KeyGroupID::from_record_and_idx(record_id, idx); 
            self.put_key_group_internal(key_group_id, &key_set)?;
        }

        //Put the key group record into the rec_data table
        let group_indices : Vec<usize> = (0..num_groups).into_iter().collect();
        self.put_key_groups(record_id, &group_indices[..])
    }

    /// Adds a new key to a KeyGroups transient structure.  Doesn't touch the DB
    /// 
    /// This function is the owner of the decision whether or not to add a key to an existing
    /// group or to create a new group for a key
    fn add_key_to_groups<'a, K : Key<'a, KeyCharT>>(&self, key : &K, groups : &mut KeyGroups<<Self as TableKeyEncoding<KeyCharT>>::OwnedKeyT>, update_reverse_map : bool) -> Result<(), String> {
        
        //Make sure the key is within the maximum allowable MAX_KEY_LENGTH
        if key.num_chars() > MAX_KEY_LENGTH {
            return Err("key length exceeds MAX_KEY_LENGTH".to_string());
        }

        //Compute the variants for the key
        let key_variants = self.variants(key);

        //Variables that determine which group we merge into, or whether we create a new key group
        let mut group_idx; //The index of the key group we'll merge this key into
        let create_new_group;

        //If we already have exactly this key as a variant, then we will add the key to that
        // key group
        if let Some(existing_group) = groups.variant_reverse_lookup_map.get(key.as_bytes()) {
            group_idx = *existing_group;
            create_new_group = false;
        } else {

            if self.config.group_variant_overlap_threshold > 0 {

                //Count the number of overlapping variants the key has with each existing group
                // NOTE: It's possible the variant_reverse_lookup_map doesn't capture all of the
                // different groups containing a given variant.  This could happen if we chose not
                // to merge variants for any reason, like exceeding a max number of keys in a key group,
                // It could also happen if a variant set ends up overlapping two previously disjoint
                // variant sets.  The only way to avoid that would be to merge the two existing key
                // groups into a single key group, but we don't have logic to merge existing key groups,
                // only to append new keys and the key's variants to a group.
                // Since the whole key groups logic is just an optimization, this edge case will not
                // affect the correctness of the results.
                let mut overlap_counts : Vec<usize> = vec![0; groups.key_group_keys.len()];
                for variant in key_variants.iter() {
                    //See if it's already part of another key's variant list
                    if let Some(existing_group) = groups.variant_reverse_lookup_map.get(&variant[..]) {
                        overlap_counts[*existing_group] += 1;
                    }
                }

                let (max_group_idx, max_overlaps) = overlap_counts.into_iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                .unwrap_or((0, 0));
                group_idx = max_group_idx;
                create_new_group = max_overlaps < self.config.group_variant_overlap_threshold; //Unless we have at least group_variant_overlap_threshold variant overlaps we'll make a new key group.

            } else {
                group_idx = 0;
                create_new_group = groups.key_group_keys.len() == 0;
            }
        }

        //Make a decision about whether to:
        //A.) Use the key as the start of a new key group, or
        //B.) Combine the key and its variant into an existing group
        if create_new_group {
            //A. We have no overlap with any existing group, so we will create a new group for this key
            group_idx = groups.key_group_keys.len();
            let mut new_set = HashSet::with_capacity(1);
            new_set.insert(Self::owned_key_from_key(key));
            groups.key_group_keys.push(new_set);
            groups.key_group_variants.push(key_variants.clone());
            //We can't count on the KeyGroupIDs not having holes so we need to use a function to
            //find a unique ID.
            let new_group_id = groups.next_available_group_id();
            groups.group_ids.push(new_group_id);
        } else {
            //B. We will append the key to the existing group at group_index, and merge the variants
            groups.key_group_keys[group_idx].insert(Self::owned_key_from_key(key));
            groups.key_group_variants[group_idx].extend(key_variants.clone());
        }

        //If we're not at the last key in the list, add the variants to the variant_reverse_lookup_map
        if update_reverse_map {
            for variant in key_variants {
                groups.variant_reverse_lookup_map.insert(variant, group_idx);
            }
        }

        Ok(())
    }

    /// Divides a list of keys up into one or more key groups based on some criteria; the primary
    /// of which is the overlap between key variants.  Keys with more overlapping variants are more
    /// likely to belong in the same group and keys with fewer or none are less likely.
    fn make_groups_from_keys<'a, K : Key<'a, KeyCharT> + 'a, KeysIterT : Iterator<Item=&'a K>>(&self, keys_iter : KeysIterT, num_keys : usize) -> Result<KeyGroups<<Self as TableKeyEncoding<KeyCharT>>::OwnedKeyT>, String> {

        //Start with empty key groups, and add the keys one at a time
        let mut groups = KeyGroups::new();
        for (key_idx, key) in keys_iter.enumerate() {
            let update_reverse_map = key_idx < num_keys-1;
            self.add_key_to_groups(key, &mut groups, update_reverse_map)?;
        }

        Ok(groups)
    }

    /// Loads the existing key groups for a record in the [Table]
    /// 
    /// This function is used when adding new keys to a record, and figuring out which groups to
    /// merge the keys into
    fn load_key_groups(&self, record_id : RecordID) -> Result<KeyGroups<<Self as TableKeyEncoding<KeyCharT>>::OwnedKeyT>, String> {

        let mut groups = KeyGroups::new();

        //Load the group indices from the rec_data table and loop over each key group
        for (group_idx, key_group) in self.get_key_groups(record_id)?.enumerate() {

            let mut group_keys = HashSet::new();
            let mut group_variants = HashSet::new();

            //Load the group's keys and loop over each one
            for key in self.get_keys_in_group(key_group)? {

                //Compute the variants for the key, and merge them into the group variants
                let key_variants = self.variants(&key);

                //Update the reverse_lookup_map with every variant
                for variant in key_variants.iter() {
                    groups.variant_reverse_lookup_map.insert(variant.clone(), group_idx);
                }

                //Push this key into the group's key list
                group_keys.insert(key);

                //Merge this key's variants with the other variants in this group
                group_variants.extend(key_variants);
            }

            groups.key_group_variants.push(group_variants);
            groups.key_group_keys.push(group_keys);
            groups.group_ids.push(key_group.group_idx());
        }

        Ok(groups)
    }

    /// Adds the KeyGroupID to each of the supplied variants
    fn put_variants_internal(&mut self, key_group : KeyGroupID, variants : HashSet<Vec<u8>>) -> Result<(), String> {

        //Add the key_group to each variant
        let variants_cf_handle = self.db.cf_handle(VARIANTS_CF_NAME).unwrap();
        for variant in variants {
            //TODO: Benchmark using merge_cf() against using a combination of get_pinned_cf() and put_cf()
            let val_bytes = Self::new_variant_vec(key_group);
            self.db.merge_cf(variants_cf_handle, variant, val_bytes)?;
        }

        Ok(())
    }

    /// Add additional keys to a record, including creation of all associated variants
    fn add_keys_internal<'a, K : Key<'a, KeyCharT> + 'a, KeysIterT : Iterator<Item=&'a K>>(&mut self, record_id : RecordID, keys_iter : KeysIterT, num_keys : usize) -> Result<(), String> {

        //Get the record's existing key groups and variants, so we can figure out the
        //best places for each additional new key
        let mut groups = self.load_key_groups(record_id)?;

        //Clone the existing groups, so we can determine which variants were added where
        let existing_groups_variants = groups.key_group_variants.clone();

        //Go over each key and add it to the key groups,
        // This add_key_to_groups function encapsulates the logic to add each key to
        // the correct group or create a new group
        for (key_idx, key) in keys_iter.enumerate() {
            let update_reverse_index = key_idx < num_keys-1;
            self.add_key_to_groups(key, &mut groups, update_reverse_index)?;
        }

        //Go over each group, work out the variants we need to add, then add them and update the group
        let empty_set = HashSet::new();
        for (group_idx, keys_set) in groups.key_group_keys.iter().enumerate() {

            //Get the variants for the group, as it existed prior to adding any keys
            let existing_keys_variants = match existing_groups_variants.get(group_idx) {
                Some(variant_set) => variant_set,
                None => &empty_set
            };

            //Calculate the set of variants that is unique to the keys we'll be adding
            let mut unique_keys_variants = HashSet::new();
            for unique_keys_variant in groups.key_group_variants[group_idx].difference(&existing_keys_variants) {
                unique_keys_variants.insert(unique_keys_variant.to_owned());
            }

            //Add the key_group_id to the appropriate entries for each of the new variants
            let key_group_id = KeyGroupID::from_record_and_idx(record_id, groups.group_ids[group_idx]);
            self.put_variants_internal(key_group_id, unique_keys_variants)?;

            //Add the new keys to the key group's entry in the keys table by replacing the keys vector
            // with the superset
            self.put_key_group_internal(key_group_id, keys_set)?;
        }

        //Put the new rec_data entry, to reflect all the associated key groups
        self.put_key_groups(record_id, &groups.group_ids[..])
    }

    /// Removes the specified keys from the keys associated with a record
    /// 
    /// If one of the specified keys is not associated with the record then that specified
    /// key will be ignored.
    fn remove_keys_internal<'a, K : Key<'a, KeyCharT>>(&mut self, record_id : RecordID, remove_keys : &HashSet<&K>) -> Result<(), String> {

        //Get all of the existing groups
        let group_ids : Vec<KeyGroupID> = self.get_key_groups(record_id)?.collect();

        //Go through each existing group, and build a HashSet containing the keys that
        // we will delete and the keys that will remain after the removal
        let mut remaining_key_count = 0;
        let mut deleted_group_keys_sets = vec![];
        let mut remaining_group_keys_sets = vec![];
        for key_group in group_ids.iter() {
            let mut deleted_keys = HashSet::new();
            let mut remaining_keys = HashSet::new();
            for existing_key in self.get_keys_in_group(*key_group)? {

                //NOTE: We know this is safe because the unsafety comes from the fact that
                // query_key might borrow existing_key, which is temporary, while query_key's
                // lifetime is 'a, which is beyond this function.  However, query_key doesn't
                // outlast this unsafe scope and we know HashSet::contains won't hold onto it.
                let set_contains_key = unsafe {
                    let query_key = K::from_owned_unsafe(&existing_key);
                    remove_keys.contains(&query_key)
                };
                
                if !set_contains_key {
                    remaining_keys.insert(existing_key);
                    remaining_key_count += 1;
                } else {
                    deleted_keys.insert(existing_key);
                }
            }
            deleted_group_keys_sets.push(deleted_keys);
            remaining_group_keys_sets.push(remaining_keys);
        }

        //If we're left with no remaining keys, we should throw an error because all records must
        //have at least one key
        if remaining_key_count < 1 {
            return Err("cannot remove all keys from record".to_string());
        }

        //Go through each group and update its keys and variants, or remove the group altogether
        let mut remaining_group_indices = vec![];
        for (idx, group_id) in group_ids.into_iter().enumerate() {

            //If we didn't remove any keys from this group, there is nothing to do here.
            if deleted_group_keys_sets[idx].len() == 0 {
                remaining_group_indices.push(group_id.group_idx());
                continue;
            }

            //Compute all variants for the keys we're removing from this group
            let mut remove_keys_variants = HashSet::new();
            for remove_key in deleted_group_keys_sets[idx].iter() {
                let keys_variants = self.variants(remove_key);
                remove_keys_variants.extend(keys_variants);
            }

            //Compute all the variants for the keys that must remain in the group
            let mut remaining_keys_variants = HashSet::new();
            for remaining_key in remaining_group_keys_sets[idx].iter() {
                let keys_variants = self.variants(remaining_key);
                remaining_keys_variants.extend(keys_variants);
            }

            //Exclude all of the overlapping variants, leaving only the variants that are unique
            //to the keys we're removing
            let mut unique_keys_variants = HashSet::new();
            for unique_keys_variant in remove_keys_variants.difference(&remaining_keys_variants) {
                unique_keys_variants.insert(unique_keys_variant.to_owned());
            }

            //Delete our KeyGroupID from each variant's list, and remove the variant list if we made it empty
            self.delete_variants_internal(group_id, unique_keys_variants)?;

            //Update or delete the group
            if remaining_group_keys_sets[idx].len() == 0 {
                //Delete the group's keys record if we made the group empty
                self.delete_key_group_entry(group_id)?;
            } else {
                //Otherwise update the group's keys record
                self.put_key_group_internal(group_id, &remaining_group_keys_sets[idx])?;
                remaining_group_indices.push(group_id.group_idx());
            }
            
        }

        //Update the record's rec_data entry to reflect the new groups after deletion
        self.put_key_groups(record_id, &remaining_group_indices[..])
    }

    /// Replaces all of the keys in a record with the supplied keys
    fn replace_keys_internal<'a, K : Key<'a, KeyCharT>>(&mut self, record_id : RecordID, keys : &'a [K]) -> Result<(), String> {

        if keys.len() < 1 {
            return Err("record must have at least one key".to_string());
        }
        if keys.iter().any(|key| key.num_chars() > MAX_KEY_LENGTH) {
            return Err("key length exceeds MAX_KEY_LENGTH".to_string());
        }

        //Delete the old keys
        self.delete_keys_internal(record_id)?;

        //Set the keys on the new record
        self.put_record_keys(record_id, keys.into_iter(), keys.len())
    }

    /// Deletes a record's value in the values table
    /// 
    /// This should only be called as part of another operation as it leaves the record in an
    /// inconsistent state
    fn delete_value_internal(&mut self, record_id : RecordID) -> Result<(), String> {

        let value_cf_handle = self.db.cf_handle(VALUES_CF_NAME).unwrap();
        self.db.delete_cf(value_cf_handle, usize::to_le_bytes(record_id.0))?;

        Ok(())
    }

    /// Creates entries in the values table
    /// If we are updating an old record, we will overwrite it.
    /// 
    /// NOTE: This function will NOT update any variants used to locate the key
    fn put_value_internal(&mut self, record_id : RecordID, value : &ValueT) -> Result<(), String> {
        
        //Serialize the value and put it in the values table.
        let value_cf_handle = self.db.cf_handle(VALUES_CF_NAME).unwrap();
        let record_coder = bincode::DefaultOptions::new().with_varint_encoding().with_little_endian();
        let value_bytes = record_coder.serialize(value).unwrap();
        self.db.put_cf(value_cf_handle, usize::to_le_bytes(record_id.0), value_bytes)?;

        Ok(())
    }

    /// Replaces a record's value with the supplied value.  Returns the value that was replaced
    /// 
    /// The supplied `record_id` must references an existing record that has not been deleted.
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn replace_value(&mut self, record_id : RecordID, value : &ValueT) -> Result<ValueT, String> {

        let old_value = self.get_value(record_id)?;

        self.put_value_internal(record_id, value)?;

        Ok(old_value)
    }

    /// Inserts a record into the Table, called by insert(), which is implemented differently depending
    /// on the UTF8_KEYS constant
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    fn insert_internal<'a, K : Key<'a, KeyCharT> + 'a, KeysIterT : Iterator<Item=&'a K>>(&mut self, keys_iter : KeysIterT, num_keys : usize, value : &ValueT) -> Result<RecordID, String> {

        if num_keys < 1 {
            return Err("record must have at least one key".to_string());
        }

        let new_record_id = match self.deleted_records.pop() {
            None => {
                //We'll be creating a new record, so get the next unique record_id
                let new_record_id = RecordID(self.record_count);
                self.record_count += 1;
                new_record_id
            },
            Some(record_id) => record_id
        };

        //Set the keys on the new record
        self.put_record_keys(new_record_id, keys_iter, num_keys)?;

        //Put the value into its appropriate table
        self.put_value_internal(new_record_id, value)?;

        Ok(new_record_id)
    }

    /// Visits all possible candidate keys for a given fuzzy search key, based on config.max_deletes,
    /// and invokes the supplied closure for each candidate KeyGroup found.
    /// 
    /// NOTE: The same group may be found via multiple variants.  It is the responsibility of
    /// the closure to avoid doing duplicate work.
    /// 
    /// QUESTION: should the visitor closure be able to return a bool, to mean "stop" or "keep going"?
    fn visit_fuzzy_candidates<'a, K : Key<'a, KeyCharT>, F : FnMut(KeyGroupID)>(&self, key : &K, mut visitor : F) -> Result<(), String> {

        if key.num_chars() > MAX_KEY_LENGTH {
            return Err("key length exceeds MAX_KEY_LENGTH".to_string());
        }

        //Create all of the potential variants based off of the "meaningful" part of the key
        let variants = self.variants(key);

        //Check to see if we have entries in the "variants" database for any of the key variants
        let variants_cf_handle = self.db.cf_handle(VARIANTS_CF_NAME).unwrap();
        for variant in variants {
            // See if we have an entry in the variants database
            if let Some(variant_vec_bytes) = self.db.get_pinned_cf(variants_cf_handle, variant)? {

                #[cfg(feature = "perf_counters")]
                {
                    let num_key_group_ids = bincode_vec_fixint_len(&variant_vec_bytes);
                    let mut perf_counters = self.perf_counters.get();
                    perf_counters.variant_load_count += 1;
                    if perf_counters.max_variant_entry_refs < num_key_group_ids {
                        perf_counters.max_variant_entry_refs = num_key_group_ids;
                    }
                    self.perf_counters.set(perf_counters);
                }
        
                // Call the visitor for each KeyGroup we found
                for key_group_id_bytes in bincode_vec_iter::<KeyGroupID>(&variant_vec_bytes) {
                    visitor(KeyGroupID(usize::from_le_bytes(key_group_id_bytes.try_into().unwrap())));
                }
            }
        }

        Ok(())
    }

    fn lookup_fuzzy_raw_internal<'a, K : Key<'a, KeyCharT>>(&self, key : &K) -> Result<impl Iterator<Item=RecordID>, String> {

        //Create a new HashSet to hold all of the RecordIDs that we find
        let mut result_set = HashSet::new(); //TODO, may want to allocate this with a non-zero capacity

        //Our visitor closure just puts the KeyGroup's RecordID into a HashSet
        let raw_visitor_closure = |key_group_id : KeyGroupID| {
            result_set.insert(key_group_id.record_id());
        };

        //Visit all the potential records
        self.visit_fuzzy_candidates(key, raw_visitor_closure)?;

        //Return an iterator through the HashSet we just made
        Ok(result_set.into_iter())
    }

    /// Returns an iterator over all RecordIDs and smallest distance values found with a fuzzy lookup
    /// after evaluating the supplied  distance function for every found candidate key.
    /// 
    /// NOTE: This function evaluates the distance function for all keys in advance of returning the
    /// iterator.  A lazy evaluation would be possible but would incur a sort of the KeyGroupIDs.
    /// This would be needed to ensure that the smallest distance value for a given record was returned.
    /// It would be necessary to evaluate every key group for a particular record before returning the
    /// record.  The decision not to do this is on account of the fact that [lookup_fuzzy_raw_internal]
    /// could be used instead if the caller wants a quick-to-return iterator.
    fn lookup_fuzzy_internal<'a, K : Key<'a, KeyCharT>>(&self, key : &K, threshold : Option<DistanceT>) -> Result<impl Iterator<Item=(RecordID, DistanceT)>, String> {

        let distance_function = self.config.distance_function;

        //Create a new HashMap to hold all of the RecordIDs that we might want to return, and the lowest
        // distance we find for that particular record
        let mut result_map = HashMap::new(); //TODO, may want to allocate this with a non-zero capacity
        let mut visited_groups = HashSet::new();

        //If we can borrow the lookup chars directly then do it, otherwise get them from a buffer
        let key_chars_vec;
        let looup_key_chars = if let Some(key_chars) = key.borrow_key_chars() {
            key_chars
        } else {
            key_chars_vec = key.get_key_chars();
            &key_chars_vec[..]
        };

        //pre-allocate the buffer we'll expand the key-chars into
        let mut key_chars_buf : Vec<KeyCharT> = Vec::with_capacity(MAX_KEY_LENGTH);

        //Our visitor closure tests the key using the distance function and threshold
        let lookup_fuzzy_visitor_closure = |key_group_id : KeyGroupID| {

            //QUESTION: Should we have an alternate fast path that only evaluates until we find
            // any distance smaller than threshold?  It would mean we couldn't return a reliable
            // distance but would save us evaluating distance for potentially many keys
            
            if !visited_groups.contains(&key_group_id) {

                //Check the record's keys with the distance function and find the smallest distance
                let mut record_keys_iter = self.get_keys_in_group(key_group_id).unwrap().into_iter();
                let record_key = record_keys_iter.next().unwrap(); //If we have a zero-element keys array, it's a bug elsewhere, so this unwrap should always succeed
                let record_key_chars = Self::owned_key_into_buf(&record_key, &mut key_chars_buf);
                let mut smallest_distance = distance_function(&record_key_chars[..], &looup_key_chars[..]); 
                for record_key in record_keys_iter {
                    let record_key_chars = Self::owned_key_into_buf(&record_key, &mut key_chars_buf);
                    let distance = distance_function(&record_key_chars, &looup_key_chars[..]);
                    if distance < smallest_distance {
                        smallest_distance = distance;
                    }
                }

                match threshold {
                    Some(threshold) => {
                        if smallest_distance <= threshold{
                            result_map.insert(key_group_id.record_id(), smallest_distance);
                        }       
                    }
                    None => {
                        result_map.insert(key_group_id.record_id(), smallest_distance);
                    }
                }

                //Record that we've visited this key group, we we can skip it if we encounter it
                //via a different variant
                visited_groups.insert(key_group_id);
            }
        };

        //Visit all the potential records
        self.visit_fuzzy_candidates(key, lookup_fuzzy_visitor_closure)?;

        //Return an iterator through the HashSet we just made
        Ok(result_map.into_iter())
    }

    fn lookup_best_internal<'a, K : Key<'a, KeyCharT>>(&self, key : &K) -> Result<impl Iterator<Item=RecordID>, String> {

        //First, we should check to see if lookup_exact gives us what we want.  Because if it does,
        // it's muuuuuuch faster.  If we have an exact result, no other key will be a better match
        let mut results_vec = self.lookup_exact_internal(key)?;
        if results_vec.len() > 0 {
            return Ok(results_vec.into_iter());
        }
        
        //Assuming lookup_exact didn't work, we'll need to perform the whole fuzzy lookup and iterate each key
        //to figure out the closest distance
        let mut result_iter = self.lookup_fuzzy_internal(key, None)?;
        
        if let Some(first_result) = result_iter.next() {
            let mut best_distance = first_result.1;
            results_vec.push(first_result.0);
            for result in result_iter {
                if result.1 == best_distance {
                    results_vec.push(result.0);
                } else if result.1 < best_distance {
                    //We've found a shorter distance, so drop the results_vec and start a new one
                    best_distance = result.1;
                    results_vec = vec![];
                    results_vec.push(result.0);
                }
            }

            return Ok(results_vec.into_iter());
        }

        Ok(vec![].into_iter())
    }

    /// Checks the table for records with keys that precisely match the key supplied
    /// 
    /// This function will be more efficient than a fuzzy lookup.
    fn lookup_exact_internal<'a, K : Key<'a, KeyCharT>>(&self, lookup_key : &K) -> Result<Vec<RecordID>, String> {

        let lookup_key_len = lookup_key.num_chars();
        if lookup_key_len > MAX_KEY_LENGTH {
            return Err("key length exceeds MAX_KEY_LENGTH".to_string());
        }

        let meaningful_key = self.meaningful_key_substring(lookup_key);
        let meaningful_noop = meaningful_key.num_chars() == lookup_key_len;

        //Get the variant for our meaningful_key
        let variants_cf_handle = self.db.cf_handle(VARIANTS_CF_NAME).unwrap();
        if let Some(variant_vec_bytes) = self.db.get_pinned_cf(variants_cf_handle, meaningful_key.as_bytes())? {

            let record_ids : Vec<RecordID> = if meaningful_noop {

                //If the meaningful_key exactly equals our key, we can just return the variant's results
                bincode_vec_iter::<KeyGroupID>(&variant_vec_bytes).map(|key_group_id_bytes| {
                    KeyGroupID(usize::from_le_bytes(key_group_id_bytes.try_into().unwrap()))
                }).map(|key_group_id| key_group_id.record_id()).collect()

            } else {

                //But if they are different, we need to Iterate every KeyGroupID in the variant in order
                //  to check if we really have a match on the whole key
                let owned_lookup_key = Self::owned_key_from_key(lookup_key);
                bincode_vec_iter::<KeyGroupID>(&variant_vec_bytes)
                .filter_map(|key_group_id_bytes| {
                    let key_group_id = KeyGroupID(usize::from_le_bytes(key_group_id_bytes.try_into().unwrap()));
                    
                    // Return only the KeyGroupIDs for records if their keys match the key we are looking up
                    let mut keys_iter = self.get_keys_in_group(key_group_id).ok()?;
                    if keys_iter.any(|key| key == owned_lookup_key) {
                        Some(key_group_id)
                    } else {
                        None
                    }
                }).map(|key_group_id| key_group_id.record_id()).collect()
            };

            Ok(record_ids)

        } else {

            //No variant found, so return an empty Iterator
            Ok(vec![])
        }
    }

    /// Returns the value associated with the specified record
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn get_value(&self, record_id : RecordID) -> Result<ValueT, String> {

        //Get the value object by deserializing the bytes from the db
        let values_cf_handle = self.db.cf_handle(VALUES_CF_NAME).unwrap();
        if let Some(value_bytes) = self.db.get_pinned_cf(values_cf_handle, record_id.0.to_le_bytes())? {
            let record_coder = bincode::DefaultOptions::new().with_varint_encoding().with_little_endian();
            let value : ValueT = record_coder.deserialize(&value_bytes).unwrap();

            Ok(value)
        } else {
            Err("Invalid record_id".to_string())
        }
    }

    /// Returns the number of keys associated with a specified record
    pub fn keys_count(&self, record_id : RecordID) -> Result<usize, String> {

        let mut keys_count = 0;

        //Go over every key group associated with the record
        for key_group in self.get_key_groups(record_id)? {

            let keys_cf_handle = self.db.cf_handle(KEYS_CF_NAME).unwrap();
            if let Some(keys_vec_bytes) = self.db.get_pinned_cf(keys_cf_handle, key_group.0.to_le_bytes())? {
    
                //The vector element count should be the first encoded usize
                let mut skip_bytes = 0;
                let group_keys_count = bincode_u64_le_varint(&keys_vec_bytes, &mut skip_bytes);
    
                keys_count += group_keys_count;
            } else {
                panic!(); //If we hit this, we have a corrupt DB
            }
        }

        Ok(keys_count as usize)
    }

    /// Returns an iterator for every key group associated with a specified record
    fn get_key_groups(&self, record_id : RecordID) -> Result<impl Iterator<Item=KeyGroupID>, String> {

        let rec_data_cf_handle = self.db.cf_handle(RECORD_DATA_CF_NAME).unwrap();
        if let Some(rec_data_vec_bytes) = self.db.get_pinned_cf(rec_data_cf_handle, record_id.0.to_le_bytes())? {
            let record_coder = bincode::DefaultOptions::new().with_varint_encoding().with_little_endian();
            let rec_data : RecordData = record_coder.deserialize(&rec_data_vec_bytes).unwrap();

            if rec_data.key_groups.len() > 0 {
                Ok(rec_data.key_groups.into_iter().map(move |group_idx| KeyGroupID::from_record_and_idx(record_id, group_idx)))
            } else {
                Err("Invalid record_id".to_string())
            }
        } else {
            Err("Invalid record_id".to_string())
        }
    }

    /// Replaces the key groups in the specified record with the provided vec
    fn put_key_groups(&self, record_id : RecordID, key_groups_vec : &[usize]) -> Result<(), String> {

        //Create the RecordData, serialize it, and put in into the rec_data table.
        let rec_data_cf_handle = self.db.cf_handle(RECORD_DATA_CF_NAME).unwrap();
        let record_coder = bincode::DefaultOptions::new().with_varint_encoding().with_little_endian();
        let new_rec_data = RecordData::new(key_groups_vec);
        let rec_data_bytes = record_coder.serialize(&new_rec_data).unwrap();
        self.db.put_cf(rec_data_cf_handle, usize::to_le_bytes(record_id.0), rec_data_bytes)?;

        Ok(())
    }

    /// Returns all of the keys for a record, across all key groups
    fn get_keys_internal<'a>(&'a self, record_id : RecordID) -> Result<impl Iterator<Item=<Self as TableKeyEncoding<KeyCharT>>::OwnedKeyT> + 'a, String> {

        let key_groups_iter = self.get_key_groups(record_id)?;
        let result_iter = key_groups_iter.flat_map(move |key_group| self.get_keys_in_group(key_group).unwrap());

        Ok(result_iter)
    }

    /// Returns the keys associated with a single key group of a single specified record
    fn get_keys_in_group(&self, key_group : KeyGroupID) -> Result<impl Iterator<Item=<Self as TableKeyEncoding<KeyCharT>>::OwnedKeyT>, String> {

        //Get the keys vec by deserializing the bytes from the db
        let keys_cf_handle = self.db.cf_handle(KEYS_CF_NAME).unwrap();
        if let Some(keys_vec_bytes) = self.db.get_pinned_cf(keys_cf_handle, key_group.0.to_le_bytes())? {
            let record_coder = bincode::DefaultOptions::new().with_varint_encoding().with_little_endian();
            let keys_vec : Vec<<Self as TableKeyEncoding<KeyCharT>>::OwnedKeyT> = record_coder.deserialize(&keys_vec_bytes).unwrap();

            #[cfg(feature = "perf_counters")]
            {
                let mut perf_counters = self.perf_counters.get();
                perf_counters.key_group_lookup_count += 1;
                perf_counters.keys_found_count += keys_vec.len();
                self.perf_counters.set(perf_counters);
            }    

            if keys_vec.len() > 0 {
                Ok(keys_vec.into_iter())
            } else {
                Err("Invalid record_id".to_string())
            }
        } else {
            Err("Invalid record_id".to_string())
        }
    }

    // Creates a Vec<KeyGroupID> with one entry, serialized out as a string of bytes
    fn new_variant_vec(key_group : KeyGroupID) -> Vec<u8> {

        //Create a new vec and Serialize it out
        let new_vec = vec![key_group];
        let vec_coder = bincode::DefaultOptions::new().with_fixint_encoding().with_little_endian();
        vec_coder.serialize(&new_vec).unwrap()
    }

    // The function to add a new entry for a variant in the database, formulated as a RocksDB callback
    fn variant_append_merge(_key: &[u8], existing_val: Option<&[u8]>, operands: &mut MergeOperands) -> Option<Vec<u8>> {

        // Note: I've seen this function be called at odd times by RocksDB, such as when a DB is
        // opened.  I haven't been able to get a straight answer on why RocksDB calls this function
        // unnecessarily, but it doesn't seem to be hurting performance much.

        //TODO: Status prints in this function to understand the behavior of RocksDB.
        // Remove them when this is understood.
        // println!("Append-Called {:?}", std::str::from_utf8(key).unwrap());
        let vec_coder = bincode::DefaultOptions::new().with_fixint_encoding().with_little_endian();

        //Deserialize the existing database entry into a vec of KeyGroupIDs
        //NOTE: we're actually using a HashSet because we don't want any duplicates
        let mut variant_vec = if let Some(existing_bytes) = existing_val {
            let new_vec : HashSet<KeyGroupID> = vec_coder.deserialize(existing_bytes).unwrap();
            new_vec
        } else {
            //TODO: Remove status println!()
            // println!("MERGE WITH NONE!!");
            HashSet::with_capacity(operands.size_hint().0)
        };

        //Add the new KeyGroupID(s)
        for op in operands {
            //Deserialize the vec on the operand, and merge its entries into the existing vec
            let operand_vec : HashSet<KeyGroupID> = vec_coder.deserialize(op).unwrap();
            variant_vec.extend(operand_vec);
        }

        //TODO: Remove status println!()
        // println!("AppendResults {:?}", variant_vec);

        //Serialize the vec back out again
        let result = vec_coder.serialize(&variant_vec).unwrap();
        Some(result)
    }

    // Returns the "meaningful" part of a key, that is used as the starting point to generate the variants
    fn meaningful_key_substring<'a, K : Key<'a, KeyCharT>>(&self, key: &K) -> <Self as TableKeyEncoding<KeyCharT>>::OwnedKeyT {
        if UTF8_KEYS {
            let result_string = unicode_truncate(key.borrow_key_str().unwrap(), self.config.meaningful_key_len);
            Self::owned_key_from_string(result_string)
        } else {
            let result_vec = if key.num_chars() > self.config.meaningful_key_len {
                let (prefix, _remainder) = key.borrow_key_chars().unwrap().split_at(self.config.meaningful_key_len);
                prefix.to_vec()
            } else {
                key.get_key_chars()
            };
            Self::owned_key_from_vec(result_vec)
        }
    }

    // Returns a new owned key, that is a variant of the supplied key, without the character at the
    // specified index
    fn remove_char_from_key<'a, K : Key<'a, KeyCharT>>(key: &K, idx : usize) -> <Self as TableKeyEncoding<KeyCharT>>::OwnedKeyT {
        if UTF8_KEYS {
            let result_string = unicode_remove_char(key.borrow_key_str().unwrap(), idx);
            Self::owned_key_from_string(result_string)
        } else {
            let mut result_vec = key.get_key_chars();
            result_vec.remove(idx);
            Self::owned_key_from_vec(result_vec)
        }
    }

    /// Returns all of the variants of a key, for querying or adding to the variants database
    fn variants<'a, K : Key<'a, KeyCharT>>(&self, key: &K) -> HashSet<Vec<u8>> {

        let mut variants_set : HashSet<Vec<u8>> = HashSet::new();
        
        //We shouldn't make any variants for empty keys
        if key.num_chars() > 0 {

            //We'll only build variants from the meaningful portion of the key
            let meaningful_key = self.meaningful_key_substring(key);

            if 0 < self.config.max_deletes {
                self.variants_recursive(&meaningful_key, 0, &mut variants_set);
            }
            variants_set.insert(meaningful_key.into_bytes());    
        }

        variants_set
    }
    
    // The recursive part of the variants() function
    fn variants_recursive<'a, K : Key<'a, KeyCharT>>(&self, key: &K, edit_distance: usize, variants_set: &mut HashSet<Vec<u8>>) {
    
        let edit_distance = edit_distance + 1;
    
        let key_len = key.num_chars();
    
        if key_len > 1 {
            for i in 0..key_len {
                let variant = Self::remove_char_from_key(key, i);
    
                if !variants_set.contains(variant.as_bytes()) {
    
                    if edit_distance < self.config.max_deletes {
                        self.variants_recursive(&variant, edit_distance, variants_set);
                    }
    
                    variants_set.insert(variant.into_bytes());
                }
            }
        }
    }

    #[cfg(feature = "perf_counters")]
    pub fn reset_perf_counters(&self) {
        self.perf_counters.set(PerfCounters::new());
    }
}

impl <DistanceT : 'static + Copy + Zero + PartialOrd + PartialEq + From<u8>, ValueT : 'static + Serialize + serde::de::DeserializeOwned>Table<char, DistanceT, ValueT, true> {

    /// Inserts a new key-value pair into the table and returns the RecordID of the new record
    /// 
    /// This is a high-level interface to be used if multiple keys are not needed, but is
    /// functions the same as [create](Table::create)
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn insert(&mut self, key : &str, value : &ValueT) -> Result<RecordID, String> {
        self.insert_internal([&key].iter().map(|key| *key), 1, value)
    }

    /// Retrieves a key-value pair using a RecordID
    /// 
    /// This is a high-level interface to be used if multiple keys are not needed, but is
    /// functions the same as [get_one_key](Table::get_one_key) / [get_value](Table::get_value)
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn get(&self, record_id : RecordID) -> Result<(String, ValueT), String> {
        let key = self.get_one_key(record_id)?;
        let value = self.get_value(record_id)?;

        Ok((key, value))
    }

    /// Creates a new record in the table and returns the RecordID of the new record
    /// 
    /// This function will always create a new record, regardless of whether an identical key exists.
    /// It is permissible to have two distinct records with identical keys.
    /// 
    /// NOTE: This function takes an &T for value rather than an owned T because it must make an
    /// internal copy regardless of passed ownership, so requiring an owned object would ofter
    /// result in a redundant copy.  However this is different from most containers, and makes things
    /// feel awkward when using [String] types for values.
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn create<'a, K : Key<'a, char>>(&mut self, keys : &'a [K], value : &ValueT) -> Result<RecordID, String> {
        self.insert_internal(keys.into_iter(), keys.len(), value)
    }

    /// Adds the supplied keys to the record's keys
    /// 
    /// The supplied `record_id` must references an existing record that has not been deleted.
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn add_keys<'a, K : Key<'a, char>>(&mut self, record_id : RecordID, keys : &'a [K]) -> Result<(), String> {
        self.add_keys_internal(record_id, keys.into_iter(), keys.len())
    }

//GOATGOAT, Should I take a Key instead, for all of the above functions?????

//GOATGOATGOAT, Need a test where I pass a Vec<char> as the key to a utf8 encoded key table
// Need to test creating with utf-8 and finding using both exact and fuzzy, using a char vec
// need to test creating with a char vec, and finding both exact and fuzzy, using a utf-8 string

    /// Removes the supplied keys from the keys associated with a record
    /// 
    /// If one of the specified keys is not associated with the record then that specified
    /// key will be ignored.
    /// 
    /// If removing the keys would result in a record with no keys, this operation will return
    /// an error and no keys will be removed, because all records must have at least one key.
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn remove_keys<'a, K : Key<'a, char>>(&mut self, record_id : RecordID, keys : &[K]) -> Result<(), String> {
        let keys_set : HashSet<&K> = HashSet::from_iter(keys.into_iter());
        self.remove_keys_internal(record_id, &keys_set)
    }

    /// Replaces a record's keys with the supplied keys
    /// 
    /// The supplied `record_id` must references an existing record that has not been deleted.
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn replace_keys<'a, K : Key<'a, char>>(&mut self, record_id : RecordID, keys : &'a [K]) -> Result<(), String> {
        self.replace_keys_internal(record_id, keys)
    }

    /// Returns an iterator over all of the key associated with the specified record
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn get_keys<'a>(&'a self, record_id : RecordID) -> Result<impl Iterator<Item=String> + 'a, String> {
        self.get_keys_internal(record_id)
    }

    /// Returns one key associated with the specified record.  If the record has more than one key
    /// then which key is unspecified
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn get_one_key(&self, record_id : RecordID) -> Result<String, String> {
        //TODO: Perhaps we can speed this up in the future by avoiding deserializing all keys
        //NOTE: With the Key Groups architecture and the lazy nature of iterators, we'll now only
        // deserialize the keys in one key group. So perhaps that's good enough.  On the downside, we
        // now pull the rec_data entry to get the index of the first key group.  99 times out of 100,
        // we'd be able to sucessfully guess the key group entry by looking at entry 0, and could then
        // fall back to checking the rec_data entry if that failed.  Depends on whether we want the
        // last ounce of performance from this function or not.
        let first_key = self.get_keys_internal(record_id)?.next().unwrap();
        Ok(first_key)
    }

    /// Locates all records in the table with keys that precisely match the key supplied
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn lookup_exact(&self, key : &str) -> Result<impl Iterator<Item=RecordID>, String> {
        self.lookup_exact_internal(&key).map(|result_vec| result_vec.into_iter())
    }

    /// Locates all records in the table with a key that is within a deletion distance of [config.max_deletes] of
    /// the key supplied, based on the SymSpell algorithm.
    /// 
    /// This function underlies all fuzzy lookups, and does no further filtering based on any distance function.
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn lookup_fuzzy_raw<'a>(&'a self, key : &'a str) -> Result<impl Iterator<Item=RecordID> + 'a, String> {
        self.lookup_fuzzy_raw_internal(&key)
    }

    /// Locates all records in the table for which the supplied `distance_function` evaluates to a result smaller
    /// than the supplied `threshold` when comparing the record's key with the supplied `key`
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn lookup_fuzzy<'a>(&'a self, key : &'a str, threshold : DistanceT) -> Result<impl Iterator<Item=(RecordID, DistanceT)> + 'a, String> {
        self.lookup_fuzzy_internal(&key, Some(threshold))
    }

    /// Locates the record in the table for which the supplied `distance_function` evaluates to the lowest value
    /// when comparing the record's key with the supplied `key`.
    /// 
    /// If no matching record is found within the table's `config.max_deletes`, this method will return an error.
    /// 
    /// NOTE: If two or more results have the same returned distance value and that is the smallest value, the
    /// implementation does not specify which result will be returned.
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn lookup_best<'a>(&'a self, key : &'a str) -> Result<impl Iterator<Item=RecordID> + 'a, String> {
        self.lookup_best_internal(&key)
    }
}

impl <KeyCharT : 'static + Copy + Eq + Hash + Serialize + serde::de::DeserializeOwned, DistanceT : 'static + Copy + Zero + PartialOrd + PartialEq + From<u8>, ValueT : 'static + Serialize + serde::de::DeserializeOwned>Table<KeyCharT, DistanceT, ValueT, false> {

    /// Inserts a new key-value pair into the table and returns the RecordID of the new record
    /// 
    /// This is a high-level interface to be used if multiple keys are not needed, but is
    /// functions the same as [create](Table::create)
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn insert(&mut self, key : &[KeyCharT], value : &ValueT) -> Result<RecordID, String> {
        let keys_vec = vec![&key];
        self.insert_internal(keys_vec.into_iter(), 1, value)
    }

    /// Retrieves a key-value pair using a RecordID
    /// 
    /// This is a high-level interface to be used if multiple keys are not needed, but is
    /// functions the same as [get_one_key](Table::get_one_key) / [get_value](Table::get_value)
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn get(&self, record_id : RecordID) -> Result<(Vec<KeyCharT>, ValueT), String> {
        let key = self.get_one_key(record_id)?;
        let value = self.get_value(record_id)?;

        Ok((key, value))
    }

    /// Creates a new record in the table and returns the RecordID of the new record
    /// 
    /// This function will always create a new record, regardless of whether an identical key exists.
    /// It is permissible to have two distinct records with identical keys.
    /// 
    /// NOTE: This function takes an &T for value rather than an owned T because it must make an
    /// internal copy regardless of passed ownership, so requiring an owned object would ofter
    /// result in a redundant copy.  However this is different from most containers, and makes things
    /// feel awkward when using [String] types for values.
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn create<'a, K : Key<'a, KeyCharT>>(&mut self, keys : &'a [K], value : &ValueT) -> Result<RecordID, String> {
        let num_keys = keys.len();
        let keys_iter = keys.into_iter();
        self.insert_internal(keys_iter, num_keys, value)
    }

    /// Adds the supplied keys to the record's keys
    /// 
    /// The supplied `record_id` must references an existing record that has not been deleted.
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn add_keys<'a, K : Key<'a, KeyCharT>>(&mut self, record_id : RecordID, keys : &'a [K]) -> Result<(), String> {
        self.add_keys_internal(record_id, keys.into_iter(), keys.len())
    }

    /// Removes the supplied keys from the keys associated with a record
    /// 
    /// If one of the specified keys is not associated with the record then that specified
    /// key will be ignored.
    /// 
    /// If removing the keys would result in a record with no keys, this operation will return
    /// an error and no keys will be removed, because all records must have at least one key.
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn remove_keys<'a, K : Key<'a, KeyCharT>>(&mut self, record_id : RecordID, keys : &[K]) -> Result<(), String> {
        let keys_set : HashSet<&K> = HashSet::from_iter(keys.into_iter());
        self.remove_keys_internal(record_id, &keys_set)
    }

    /// Replaces a record's keys with the supplied keys
    /// 
    /// The supplied `record_id` must references an existing record that has not been deleted.
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn replace_keys<'a, K : Key<'a, KeyCharT>>(&mut self, record_id : RecordID, keys : &'a [K]) -> Result<(), String> {
        self.replace_keys_internal(record_id, keys)
    }

    /// Returns an iterator over all of the key associated with the specified record
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn get_keys<'a>(&'a self, record_id : RecordID) -> Result<impl Iterator<Item=Vec<KeyCharT>> + 'a, String> {
        self.get_keys_internal(record_id)
    }

    /// Returns one key associated with the specified record.  If the record has more than one key
    /// then which key is unspecified
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn get_one_key(&self, record_id : RecordID) -> Result<Vec<KeyCharT>, String> {
        //TODO: Perhaps we can speed this up in the future by avoiding deserializing all keys
        Ok(self.get_keys_internal(record_id)?.next().unwrap())
    }

    /// Locates all records in the table with keys that precisely match the key supplied
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn lookup_exact<'a>(&'a self, key : &'a [KeyCharT]) -> Result<impl Iterator<Item=RecordID> + 'a, String> {
        self.lookup_exact_internal(&key).map(|result_vec| result_vec.into_iter())
    }

    /// Locates all records in the table with a key that is within a deletion distance of `config.max_deletes` of
    /// the key supplied, based on the SymSpell algorithm.
    /// 
    /// This function underlies all fuzzy lookups, and does no further filtering based on any distance function.
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn lookup_fuzzy_raw<'a>(&'a self, key : &'a [KeyCharT]) -> Result<impl Iterator<Item=RecordID> + 'a, String> {
        self.lookup_fuzzy_raw_internal(&key)
    }

    /// Locates all records in the table for which the supplied `distance_function` evaluates to a result smaller
    /// than the supplied `threshold` when comparing the record's key with the supplied `key`
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn lookup_fuzzy<'a>(&'a self, key : &'a [KeyCharT], threshold : DistanceT) -> Result<impl Iterator<Item=(RecordID, DistanceT)> + 'a, String> {
        self.lookup_fuzzy_internal(&key, Some(threshold))
    }

    /// Locates the record in the table for which the supplied `distance_function` evaluates to the lowest value
    /// when comparing the record's key with the supplied `key`.
    /// 
    /// If no matching record is found within the table's `config.max_deletes`, this method will return an error.
    /// 
    /// NOTE: If two or more results have the same returned distance value and that is the smallest value, the
    /// implementation does not specify which result will be returned.
    /// 
    /// NOTE: [rocksdb::Error] is a wrapper around a string, so if an error occurs it will be the
    /// unwrapped RocksDB error.
    pub fn lookup_best<'a>(&'a self, key : &'a [KeyCharT]) -> Result<impl Iterator<Item=RecordID> + 'a, String> {
        self.lookup_best_internal(&key)
    }
}

impl <KeyCharT, DistanceT, ValueT, const UTF8_KEYS : bool>Drop for Table<KeyCharT, DistanceT, ValueT, UTF8_KEYS> {
    fn drop(&mut self) {
        //Close down Rocks
        self.db.flush().unwrap();
        let _ = DB::destroy(&rocksdb::Options::default(), self.path.as_str());
    }
}

// Returns the usize that is one larger than the largest key, assuming the column family contains a
// all of the smaller keys without any gaps.  If there are missing keys, the results are undefined.
//
// Implements a binary search through the possible keys, looking for the highest numbered key
// This function should resolve one bit of the key, each time through the loop, so it should loop
// at most 64 times for a 64 bit key, and likely much less because of the starting hint
fn probe_for_max_sequential_key(db : &DBWithThreadMode<rocksdb::SingleThreaded>, cf : &ColumnFamily, starting_hint : usize) -> Result<usize, rocksdb::Error> {

    let mut min = 0;
    let mut max = usize::MAX;

    //Need to adjust this constant so we don't risk overflow if we don't have 64-bit usize
    debug_assert!(::std::mem::size_of::<usize>() == 8);
    let mut guess_max = if starting_hint > 0xFFFFFFFF {
        usize::MAX
    } else if starting_hint < 1 {
        1
    } else {
        starting_hint * starting_hint
    };
    
    let mut cur_val = starting_hint;
    loop {

        //NOTE: this is an optimization to save one DB query at the cost of an extra test each loop
        //The case where max == min will result in no hit and exit at the bottom of the loop body
        if max == min {
            return Ok(cur_val)
        }

        if let Some(_value) = db.get_pinned_cf(cf, cur_val.to_le_bytes())? {
            //println!("Yes, cur_val = {}, min = {}, max = {}, guess_max = {}", cur_val, min, max, guess_max);
            min = cur_val + 1;
            if guess_max < max/2 {
                guess_max *= 2;
            } else {
                guess_max = max;
            }
        } else {
            //println!("No, cur_val = {}, min = {}, max = {}, guess_max = {}", cur_val, min, max, guess_max);
            max = cur_val;
            guess_max = max;

            if max == min {
                return Ok(cur_val)
            }    
        }

        cur_val = ((guess_max - min) / 2) + min;
    }
}

#[cfg(test)]
mod tests {
    use crate::{*};
    use std::fs;
    use std::path::PathBuf;
    use csv::ReaderBuilder;
    use serde::{Deserialize};

    #[test]
    /// This test is designed to stress the database with many thousand entries.
    ///  
    /// You may download an alternate GeoNames file in order to get a more rigorous test.  The included
    /// `geonames_megacities.txt` file is just a stub to avoid bloating the crate download.  The content
    /// of `geonames_megacities.txt` was derived from data on [http://geonames.org], and licensed under
    /// a [Creative Commons Attribution 4.0 License](https://creativecommons.org/licenses/by/4.0/legalcode)
    /// 
    /// A geonames file may be downloaded from: [http://download.geonames.org/export/dump/cities15000.zip]
    /// for the smallest file, and "cities500.zip" for the largest, depending on whether you want this
    /// to pass in the a lightweight way or the most thorough.
    fn geonames_test() {

        let mut geonames_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        geonames_file_path.push("geonames_megacities.txt");

        // Alternate geonames file
        // NOTE: Uncomment this to use a different file
        // let geonames_file_path = PathBuf::from("/path/to/file/cities500.txt");
//GOATGOATGOAT
//let geonames_file_path = PathBuf::from("/Users/admin/Downloads/Geonames.org/cities500.txt");

    
        //Create the FuzzyRocks Table, and clear out any records that happen to be hanging out
        let config = TableConfig::<char, u8, i32, true>::default();
        let mut table = Table::new("geonames.rocks", config).unwrap();
        table.reset().unwrap();

        //Data structure to parse the GeoNames TSV file into
        #[derive(Clone, Debug, Serialize, Deserialize)]
        struct GeoName {
            geonameid         : i32, //integer id of record in geonames database
            name              : String, //name of geographical point (utf8) varchar(200)
            asciiname         : String, //name of geographical point in plain ascii characters, varchar(200)
            alternatenames    : String, //alternatenames, comma separated, ascii names automatically transliterated, convenience attribute from alternatename table, varchar(10000)
            latitude          : f32, //latitude in decimal degrees (wgs84)
            longitude         : f32, //longitude in decimal degrees (wgs84)
            feature_class     : char, //see http://www.geonames.org/export/codes.html, char(1)
            feature_code      : String,//[char; 10], //see http://www.geonames.org/export/codes.html, varchar(10)
            country_code      : String,//[char; 2], //ISO-3166 2-letter country code, 2 characters
            cc2               : String, //alternate country codes, comma separated, ISO-3166 2-letter country code, 200 characters
            admin1_code       : String,//[char; 20], //fipscode (subject to change to iso code), see exceptions below, see file admin1Codes.txt for display names of this code; varchar(20)
            admin2_code       : String, //code for the second administrative division, a county in the US, see file admin2Codes.txt; varchar(80) 
            admin3_code       : String,//[char; 20], //code for third level administrative division, varchar(20)
            admin4_code       : String,//[char; 20], //code for fourth level administrative division, varchar(20)
            population        : i64, //bigint (8 byte int)
            #[serde(deserialize_with = "default_if_empty")]
            elevation         : i32, //in meters, integer
            #[serde(deserialize_with = "default_if_empty")]
            dem               : i32, //digital elevation model, srtm3 or gtopo30, average elevation of 3''x3'' (ca 90mx90m) or 30''x30'' (ca 900mx900m) area in meters, integer. srtm processed by cgiar/ciat.
            timezone          : String, //the iana timezone id (see file timeZone.txt) varchar(40)
            modification_date : String, //date of last modification in yyyy-MM-dd format
        }

        fn default_if_empty<'de, D, T>(de: D) -> Result<T, D::Error>
            where D: serde::Deserializer<'de>, T: serde::Deserialize<'de> + Default,
        {
            Option::<T>::deserialize(de).map(|x| x.unwrap_or_else(|| T::default()))
        }

        //Open the tab-saparated value file
        let tsv_file_contents = fs::read_to_string(geonames_file_path).expect("Error reading geonames file");
        let mut tsv_parser = ReaderBuilder::new()
            .delimiter(b'\t')
            .has_headers(false)
            .flexible(true) //We want to permit situations where some rows have fewer columns for now
            .quote(0)
            .double_quote(false)
            .from_reader(tsv_file_contents.as_bytes());

        //Iterate over every geoname entry in the geonames file and insert it (lowercase) into our table
        let mut record_id = RecordID::NULL;
        let mut tsv_record_count = 0;
        for geoname in tsv_parser.deserialize::<GeoName>().map(|result| result.unwrap()) {

            //Separate the comma-separated alternatenames field
            let mut names : HashSet<String> = HashSet::from_iter(geoname.alternatenames.split(',').map(|string| string.to_lowercase()));
            
            //Add the primary name for the place
            names.insert(geoname.name.to_lowercase());

            //Create a record in the table
            let names_vec : Vec<String> = names.into_iter()
                .map(|string| unicode_truncate(string.as_str(), MAX_KEY_LENGTH))
                .collect();
            record_id = table.create(&names_vec[..], &geoname.geonameid).unwrap();
            tsv_record_count += 1;

            //Status Print
            if record_id.0 % 500 == 499 {
                println!("inserting... {}, {}", geoname.name.to_lowercase(), record_id.0);
            }
        }

        //Indirectly that the number of records roughly matches the number of entries from the CSV
        //NOTE: RocksDB doesn't have a "record_count" feature, and therefore neither does our Table,
        //but since we started from a reset table, we can ensure that the last assigned record_id
        //should roughly correspond to the number of entries we inserted
        assert_eq!(record_id.0 + 1, tsv_record_count);

        //Confirm we can find a known city (London)
        let london_results : Vec<i32> = table.lookup_exact("london").unwrap().map(|record_id| table.get_value(record_id).unwrap()).collect();
        assert!(london_results.contains(&2643743)); //2643743 is the geonames_id of "London"

        //Confirm we can find a known city with a longer key name (not on the fast-path)
        let rio_results : Vec<i32> = table.lookup_exact("rio de janeiro").unwrap().map(|record_id| table.get_value(record_id).unwrap()).collect();
        assert!(rio_results.contains(&3451190)); //3451190 is the geonames_id of "Rio de Janeiro"

        //Close RocksDB connection by dropping the table object
        drop(table);
        drop(london_results);

        //Reopen the table and confirm that "London" is still there
        let config = TableConfig::<char, u8, i32, true>::default();
        let table = Table::new("geonames.rocks", config).unwrap();
        let london_results : Vec<i32> = table.lookup_exact("london").unwrap().map(|record_id| table.get_value(record_id).unwrap()).collect();
        assert!(london_results.contains(&2643743)); //2643743 is the geonames_id of "London"
    }

    #[test]
    fn fuzzy_rocks_test() {

        //Create and reset the FuzzyRocks Table
        let mut config = DEFAULT_UTF8_TABLE;
        config.meaningful_key_len = 8;
        let mut table = Table::<char, u8, String, true>::new("test.rocks", config).unwrap();
        table.reset().unwrap();

        //Insert some records
        let sun = table.insert("Sunday", &"Nichiyoubi".to_string()).unwrap();
        let sat = table.insert("Saturday", &"Douyoubi".to_string()).unwrap();
        let fri = table.insert("Friday", &"Kinyoubi".to_string()).unwrap();
        let thu = table.insert("Thursday", &"Mokuyoubi".to_string()).unwrap();
        let wed = table.insert("Wednesday", &"Suiyoubi".to_string()).unwrap();
        let tue = table.insert("Tuesday", &"Kayoubi".to_string()).unwrap();
        let mon = table.insert("Monday", &"Getsuyoubi".to_string()).unwrap();

        //Test lookup_exact
        let results : Vec<(String, String)> = table.lookup_exact("Friday").unwrap().map(|record_id| table.get(record_id).unwrap()).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "Friday");
        assert_eq!(results[0].1, "Kinyoubi");

        //Test lookup_exact, with a query that should provide no results
        let results : Vec<RecordID> = table.lookup_exact("friday").unwrap().collect();
        assert_eq!(results.len(), 0);

        //Test lookup_best, using the supplied edit_distance function
        let results : Vec<RecordID> = table.lookup_best("Bonday").unwrap().collect();
        assert_eq!(results.len(), 1);
        assert!(results.contains(&mon));

        //Test lookup_best, when there is no acceptable match
        let results : Vec<RecordID> = table.lookup_best("Rahu").unwrap().collect();
        assert_eq!(results.len(), 0);

        //Test lookup_fuzzy with a perfect match, using the supplied edit_distance function
        //In this case, we should only get one match within edit-distance 2
        let results : Vec<(String, String, u8)> = table.lookup_fuzzy("Saturday", 2)
            .unwrap().map(|(record_id, distance)| {
                let (key, val) = table.get(record_id).unwrap();
                (key, val, distance)
            }).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "Saturday");
        assert_eq!(results[0].1, "Douyoubi");
        assert_eq!(results[0].2, 0);

        //Test lookup_fuzzy with a perfect match, but where we'll hit another imperfect match as well
        let results : Vec<(String, String, u8)> = table.lookup_fuzzy("Tuesday", 2)
            .unwrap().map(|(record_id, distance)| {
                let (key, val) = table.get(record_id).unwrap();
                (key, val, distance)
            }).collect();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&("Tuesday".to_string(), "Kayoubi".to_string(), 0)));
        assert!(results.contains(&("Thursday".to_string(), "Mokuyoubi".to_string(), 2)));

        //Test lookup_fuzzy where we should get no match
        let results : Vec<(RecordID, u8)> = table.lookup_fuzzy("Rahu", 2).unwrap().collect();
        assert_eq!(results.len(), 0);

        //Test lookup_fuzzy_raw, to get all of the SymSpell Delete variants
        //We're testing the fact that characters beyond `config.meaningful_key_len` aren't used for the comparison
        let results : Vec<RecordID> = table.lookup_fuzzy_raw("Sunday. That's my fun day.").unwrap().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], sun);

        //Test deleting a record, and ensure we can't access it or any trace of its variants
        table.delete(tue).unwrap();
        assert!(table.get_one_key(tue).is_err());

        //Since "Tuesday" had one variant overlap with "Thursday", i.e. "Tusday", make sure we now find
        // "Thursday" when we attempt to lookup "Tuesday"
        let results : Vec<RecordID> = table.lookup_best("Tuesday").unwrap().collect();
        assert_eq!(results.len(), 1);
        assert!(results.contains(&thu));

        //Delete "Saturday" and make sure we see no matches when we try to search for it
        table.delete(sat).unwrap();
        assert!(table.get_one_key(sat).is_err());
        let results : Vec<RecordID> = table.lookup_fuzzy_raw("Saturday").unwrap().collect();
        assert_eq!(results.len(), 0);

        //Test replacing a record with another one and ensure the right data is retained
        table.replace_keys(wed, &["Miercoles"]).unwrap();
        table.replace_value(wed, &"Zhousan".to_string()).unwrap();
        let results : Vec<(String, String)> = table.lookup_exact("Miercoles").unwrap().map(|record_id| (table.get_one_key(record_id).unwrap(), table.get_value(record_id).unwrap())).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "Miercoles");
        assert_eq!(results[0].1, "Zhousan");
        let results : Vec<RecordID> = table.lookup_fuzzy_raw("Mercoledi").unwrap().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], wed);

        //Try replacing the keys and value on a record that we deleted earlier, and make sure
        // we get the right errors
        assert!(table.replace_keys(sat, &["Sabado"]).is_err());
        assert!(table.replace_value(sat, &"Zhouliu".to_string()).is_err());

        //Attempt to replace a record's keys with an empty list, check the error
        let empty_slice : &[&str] = &[];
        assert!(table.replace_keys(sat, empty_slice).is_err());
        
        //Attempt to replace an invalid record and confirm we get a reasonable error
        assert!(table.replace_keys(RecordID::NULL, &["Nullday"]).is_err());
        assert!(table.replace_value(RecordID::NULL, &"Null".to_string()).is_err());

        //Test that create() returns the right error if no keys are supplied
        let empty_slice : &[&str] = &[];
        assert!(table.create(empty_slice, &"Douyoubi".to_string()).is_err());

        //Recreate Saturday using the create() api
        //While we're here, Also test that the same key string occurring more than once
        // doesn't result in additional keys being added
        let sat = table.create(&["Saturday", "Saturday"], &"Douyoubi".to_string()).unwrap();
        table.add_keys(sat, &["Saturday", "Saturday"]).unwrap();
        assert_eq!(table.keys_count(sat).unwrap(), 1);

        //Add some new keys to it, and verify that it can be found using any of its three keys
        table.add_keys(sat, &["Sabado", "Zhouliu"]).unwrap();
        assert_eq!(table.keys_count(sat).unwrap(), 3);
        let results : Vec<RecordID> = table.lookup_fuzzy_raw("Saturday").unwrap().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], sat);
        let results : Vec<RecordID> = table.lookup_exact("Zhouliu").unwrap().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], sat);
        let results : Vec<RecordID> = table.lookup_fuzzy_raw("Sabato").unwrap().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], sat);

        //Test deleting one of the keys from a record, and make sure we can't find it using that
        //key but the other keys are unaffected
        table.remove_keys(sat, &["Sabado"]).unwrap();
        assert_eq!(table.keys_count(sat).unwrap(), 2);
        let results : Vec<RecordID> = table.lookup_exact("Sabado").unwrap().collect();
        assert_eq!(results.len(), 0);
        let results : Vec<RecordID> = table.lookup_fuzzy_raw("Sabato").unwrap().collect();
        assert_eq!(results.len(), 0);
        let results : Vec<RecordID> = table.lookup_fuzzy_raw("Saturnsday").unwrap().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], sat);
        let results : Vec<RecordID> = table.lookup_exact("Zhouliu").unwrap().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], sat);

        //Attempt to remove the remaining keys and ensure we get an error and both keys remain
        assert!(table.remove_keys(sat, &["Saturday", "Zhouliu"]).is_err());
        assert_eq!(table.keys_count(sat).unwrap(), 2);
        let results : Vec<RecordID> = table.lookup_exact("Saturday").unwrap().collect();
        assert_eq!(results.len(), 1);
        let results : Vec<RecordID> = table.lookup_exact("Zhouliu").unwrap().collect();
        assert_eq!(results.len(), 1);

        //Test that replacing the keys of a record doesn't leave any orphaned variants
        table.replace_keys(sat, &["Sabado"]).unwrap();
        assert_eq!(table.keys_count(sat).unwrap(), 1);
        let results : Vec<RecordID> = table.lookup_fuzzy_raw("Saturday").unwrap().collect();
        assert_eq!(results.len(), 0);
        let results : Vec<RecordID> = table.lookup_fuzzy_raw("Zhouliu").unwrap().collect();
        assert_eq!(results.len(), 0);

        //Test that adding the same key again doesn't result in multiple copies of the key
        table.add_keys(sat, &["Sabado"]).unwrap();
        assert_eq!(table.keys_count(sat).unwrap(), 1);
        table.add_keys(sat, &["Saturday", "Saturday"]).unwrap();
        assert_eq!(table.keys_count(sat).unwrap(), 2);

        //Test that nothing breaks when we have two keys with overlapping variants, and then
        // delete one
        // "Venerdi" & "Vendredi" have overlapping variant: "Venedi"
        table.add_keys(fri, &["Geumyoil", "Viernes", "Venerdi", "Vendredi"]).unwrap();
        assert_eq!(table.keys_count(fri).unwrap(), 5);
        table.remove_keys(fri, &["Vendredi"]).unwrap();
        assert_eq!(table.keys_count(fri).unwrap(), 4);
        let results : Vec<RecordID> = table.lookup_fuzzy_raw("Vendredi").unwrap().collect();
        assert_eq!(results.len(), 1); //We'll still get Venerdi as a fuzzy match

        //Try deleting the non-existent key with valid variants, to make sure nothing breaks
        table.remove_keys(fri, &["Vendredi"]).unwrap();
        let results : Vec<RecordID> = table.lookup_fuzzy_raw("Vendredi").unwrap().collect();
        assert_eq!(results.len(), 1); //We'll still get Venerdi as a fuzzy match
        assert_eq!(table.keys_count(fri).unwrap(), 4);

        //Finally delete "Venerdi", and make sure the variants are all gone
        table.remove_keys(fri, &["Venerdi"]).unwrap();
        let results : Vec<RecordID> = table.lookup_fuzzy_raw("Vendredi").unwrap().collect();
        assert_eq!(results.len(), 0);
        assert_eq!(table.keys_count(fri).unwrap(), 3);

    }

    #[test]
    /// This test is tests some basic non-unicode key functionality.
    fn non_unicode_key_test() {

        let mut config = TableConfig::<u8, u8, f32, false>::default();
        config.max_deletes = 1;
        config.meaningful_key_len = 8;
        let mut table = Table::<u8, u8, f32, false>::new("test2.rocks", config).unwrap();
        table.reset().unwrap();

        let one = table.insert(b"One", &1.0).unwrap();
        let _two = table.insert(b"Dos", &2.0).unwrap();
        let _three = table.insert(b"San", &3.0).unwrap();
        let pi = table.insert(b"Pi", &3.1415926535).unwrap();

        let results : Vec<RecordID> = table.lookup_best(b"P").unwrap().collect();
        assert_eq!(results.len(), 1);
        assert!(results.contains(&pi));
        
        let results : Vec<RecordID> = table.lookup_fuzzy_raw(b"ne").unwrap().collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], one);
    }

    #[test]
    /// This tests the perf-counters
    fn perf_counters_test() {

        //Initialize the table with a very big database
        let config = TableConfig::<char, u8, i32, true>::default();
        let table = Table::new("all_cities.geonames.rocks", config).unwrap();

        //Make sure we have no pathological case of a variant for a zero-length string
        let iter = table.lookup_fuzzy_raw("").unwrap();
        assert_eq!(iter.count(), 0);

        #[cfg(feature = "perf_counters")]
        {
            //Make sure we are on the fast path that doesn't fetch the keys for the case when the key
            //length entirely fits within config.meaningful_key_len
            table.reset_perf_counters();
            let _iter = table.lookup_exact("london").unwrap();
            assert_eq!(table.perf_counters.get().key_group_lookup_count, 0);

            //Test that the other counters do something...
            table.reset_perf_counters();
            let iter = table.lookup_fuzzy("london", 3).unwrap();
            let _ = iter.count();
            assert!(table.perf_counters.get().variant_load_count > 0);
            assert!(table.perf_counters.get().key_group_lookup_count > 0);
            assert!(table.perf_counters.get().keys_found_count > 0);
            assert!(table.perf_counters.get().max_variant_entry_refs > 0);

            //Debug Prints
            println!("variant_load_count {}", table.perf_counters.get().variant_load_count);
            println!("key_group_lookup_count {}", table.perf_counters.get().key_group_lookup_count);
            println!("keys_found_count {}", table.perf_counters.get().keys_found_count);
            println!("max_variant_entry_refs {}", table.perf_counters.get().max_variant_entry_refs);
        }
        
        #[cfg(not(feature = "perf_counters"))]
        {
            println!("perf_counters feature not enabled");
        }


    }

}

//GOATGOATGOAT
//Features since last push to crates.io:
// Multi-key support
// lookup_best now returns an iterator instead of one arbitrarily-chosen record
// Support for a generic character type in key
// Adding micro-benchmarks using criterion
// Massive Perf optimizations for lookups
//  lookup_best checks lookup_exact first before more expensive lookup_fuzzy
//  key groups mingle similar keys for a record
//  optimizations to Levenstein distance function for 3x speedup
//  value table is separate from keys in DB
// 

//GOATGOATGOAT
//Next, Add multi-key support
//√ 1.) Function to add a key to a record
//√ 2.) Function to remove a key from a record
//√ 3.) Function to insert a record with multiple keys
//√ 4.) Separate out records table into keys table and values table
//√ 5.) Separate calls to get a value and get an iterator for keys
//√ 6.) Update test to insert keys for every alternative in the Geonames test
//√ 7.) Will deprecate get_record that gets both a key and a value
//√ 8.) Function to replace all keys on a record
//√ 9.) Get rid of is_valid()
//√ 10.) provide convenience fucntion called simply "get"
//√ 11.) API that counts the number of keys that a given record has

//GOATGOATGOAT, Move "BinCode Helpers" into separate file

//GOATGOATGOAT, Do a separate test for a ValueT of size 0

//GOATGOATGOAT, Clippy, and update documentation

//GOATGOAT, Provide a convenience type alias for a table with utf8 keys

//GOATGOAT Create a random keys lookup benchmark, where I generate a random key each time

//GOAT Let Wolf Garbe know about my crate when I publish FuzzyRocks v0.2

