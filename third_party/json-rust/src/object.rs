use std::{ ptr, mem, str, slice, fmt };
use std::ops::{ Index, IndexMut, Deref };
use std::string::String;
use std::vec::Vec;

use codegen::{ DumpGenerator, Generator, PrettyGenerator };
use value::JsonValue;

const KEY_BUF_LEN: usize = 32;
static NULL: JsonValue = JsonValue::Null;

// FNV-1a implementation
//
// While the `Object` is implemented as a binary tree, not a hash table, the
// order in which the tree is balanced makes absolutely no difference as long
// as there is a deterministic left / right ordering with good spread.
// Comparing a hashed `u64` is faster than comparing `&str` or even `&[u8]`,
// for larger objects this yields non-trivial performance benefits.
//
// Additionally this "randomizes" the keys a bit. Should the keys in an object
// be inserted in alphabetical order (an example of such a use case would be
// using an object as a store for entries by ids, where ids are sorted), this
// will prevent the tree from being constructed in a way where the same branch
// of each node is always used, effectively producing linear lookup times. Bad!
//
// Example:
//
// ```
// println!("{}", hash_key(b"10000056"));
// println!("{}", hash_key(b"10000057"));
// println!("{}", hash_key(b"10000058"));
// println!("{}", hash_key(b"10000059"));
// ```
//
// Produces:
//
// ```
// 15043794053238616431  <-- 2nd
// 15043792953726988220  <-- 1st
// 15043800650308385697  <-- 4th
// 15043799550796757486  <-- 3rd
// ```
#[inline]
fn hash_key(key: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in key {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

struct Key {
    // Internal buffer to store keys that fit within `KEY_BUF_LEN`,
    // otherwise this field will contain garbage.
    pub buf: [u8; KEY_BUF_LEN],

    // Length of the key in bytes.
    pub len: usize,

    // Cached raw pointer to the key, so that we can cheaply construct
    // a `&str` slice from the `Node` without checking if the key is
    // allocated separately on the heap, or in the `key_buf`.
    pub ptr: *mut u8,

    // A hash of the key, explanation below.
    pub hash: u64,
}

impl Key {
    #[inline]
    fn new(hash: u64, len: usize) -> Self {
        Key {
            buf: [0; KEY_BUF_LEN],
            len: len,
            ptr: ptr::null_mut(),
            hash: hash
        }
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.ptr, self.len)
        }
    }

    #[inline]
    fn as_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(self.as_bytes())
        }
    }

    // The `buf` on the `Key` can only be filled after the struct
    // is already on the `Vec`'s heap (along with the `Node`).
    // For that reason it's not set in `Key::new` but only after
    // the `Node` is created and allocated.
    #[inline]
    fn attach(&mut self, key: &[u8]) {
        if self.len <= KEY_BUF_LEN {
            unsafe {
                ptr::copy_nonoverlapping(
                    key.as_ptr(),
                    self.buf.as_mut_ptr(),
                    self.len
                );
            }
            self.ptr = self.buf.as_mut_ptr();
        } else {
            let mut heap = key.to_vec();
            self.ptr = heap.as_mut_ptr();
            mem::forget(heap);
        }
    }

    // Since we store `Node`s on a vector, it will suffer from reallocation.
    // Whenever that happens, `key.ptr` for short keys will turn into dangling
    // pointers and will need to be re-cached.
    #[inline]
    fn fix_ptr(&mut self) {
        if self.len <= KEY_BUF_LEN {
            self.ptr = self.buf.as_mut_ptr();
        }
    }
}

// Implement `Sync` and `Send` for `Key` despite the use of raw pointers. The struct
// itself should be memory safe.
unsafe impl Sync for Key {}
unsafe impl Send for Key {}

// Because long keys _can_ be stored separately from the `Key` on heap,
// it's essential to clean up the heap allocation when the `Key` is dropped.
impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            if self.len > KEY_BUF_LEN {
                // Construct a `Vec` out of the `key_ptr`. Since the key is
                // always allocated from a slice, the capacity is equal to length.
                let heap = Vec::from_raw_parts(
                    self.ptr,
                    self.len,
                    self.len
                );

                // Now that we have an owned `Vec<u8>`, drop it.
                drop(heap);
            }
        }
    }
}

// Just like with `Drop`, `Clone` needs a custom implementation that accounts
// for the fact that key _can_ be separately heap allocated.
impl Clone for Key {
    fn clone(&self) -> Self {
        if self.len > KEY_BUF_LEN {
            let mut heap = self.as_bytes().to_vec();
            let ptr = heap.as_mut_ptr();
            mem::forget(heap);

            Key {
                buf: [0; KEY_BUF_LEN],
                len: self.len,
                ptr: ptr,
                hash: self.hash,
            }
        } else {
            Key {
                buf: self.buf,
                len: self.len,
                ptr: ptr::null_mut(), // requires a `fix_ptr` call after `Node` is on the heap
                hash: self.hash,
            }
        }
    }
}

#[derive(Clone)]
struct Node {
    // String-esque key abstraction
    pub key: Key,

    // Value stored.
    pub value: JsonValue,

    // Store vector index pointing to the `Node` for which `key_hash` is smaller
    // than that of this `Node`.
    // Will default to 0 as root node can't be referenced anywhere else.
    pub left: usize,

    // Same as above but for `Node`s with hash larger than this one. If the
    // hash is the same, but keys are different, the lookup will default
    // to the right branch as well.
    pub right: usize,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&(self.key.as_str(), &self.value, self.left, self.right), f)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.key.hash       == other.key.hash       &&
        self.key.as_bytes() == other.key.as_bytes() &&
        self.value          == other.value
    }
}

impl Node {
    #[inline]
    fn new(value: JsonValue, hash: u64, len: usize) -> Node {
        Node {
            key: Key::new(hash, len),
            value: value,
            left: 0,
            right: 0,
        }
    }
}

/// A binary tree implementation of a string -> `JsonValue` map. You normally don't
/// have to interact with instances of `Object`, much more likely you will be
/// using the `JsonValue::Object` variant, which wraps around this struct.
#[derive(Debug)]
pub struct Object {
    store: Vec<Node>
}

impl Object {
    /// Create a new, empty instance of `Object`. Empty `Object` performs no
    /// allocation until a value is inserted into it.
    #[inline(always)]
    pub fn new() -> Self {
        Object {
            store: Vec::new()
        }
    }

    /// Create a new `Object` with memory preallocated for `capacity` number
    /// of entries.
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Object {
            store: Vec::with_capacity(capacity)
        }
    }

    #[inline]
    fn node_at_index_mut(&mut self, index: usize) -> *mut Node {
        unsafe { self.store.as_mut_ptr().offset(index as isize) }
    }

    #[inline(always)]
    fn add_node(&mut self, key: &[u8], value: JsonValue, hash: u64) -> usize {
        let index = self.store.len();

        if index < self.store.capacity() {
            // Because we've just checked the capacity, we can avoid
            // using `push`, and instead do unsafe magic to memcpy
            // the new node at the correct index without additional
            // capacity or bound checks.
            unsafe {
                let node = Node::new(value, hash, key.len());
                self.store.set_len(index + 1);

                // To whomever gets concerned: I got better results with
                // copy than write. Difference in benchmarks wasn't big though.
                ptr::copy_nonoverlapping(
                    &node as *const Node,
                    self.store.as_mut_ptr().offset(index as isize),
                    1,
                );

                // Since the Node has been copied, we need to forget about
                // the owned value, else we may run into use after free.
                mem::forget(node);
            }

            unsafe { self.store.get_unchecked_mut(index).key.attach(key) };
        } else {
            self.store.push(Node::new(value, hash, key.len()));

            unsafe { self.store.get_unchecked_mut(index).key.attach(key) };

            // Index up to the index (old length), we don't need to fix
            // anything on the Node that just got pushed.
            for node in self.store.iter_mut().take(index) {
                node.key.fix_ptr();
            }
        }

        index
    }

    /// Insert a new entry, or override an existing one. Note that `key` has
    /// to be a `&str` slice and not an owned `String`. The internals of
    /// `Object` will handle the heap allocation of the key if needed for
    /// better performance.
    #[inline]
    pub fn insert(&mut self, key: &str, value: JsonValue) {
        self.insert_index(key, value);
    }

    pub(crate) fn insert_index(&mut self, key: &str, value: JsonValue) -> usize {
        let key = key.as_bytes();
        let hash = hash_key(key);

        if self.store.len() == 0 {
            self.store.push(Node::new(value, hash, key.len()));
            self.store[0].key.attach(key);
            return 0;
        }

        let mut node = unsafe { &mut *self.node_at_index_mut(0) };
        let mut parent = 0;

        loop {
            if hash == node.key.hash && key == node.key.as_bytes() {
                node.value = value;
                return parent;
            } else if hash < node.key.hash {
                if node.left != 0 {
                    parent = node.left;
                    node = unsafe { &mut *self.node_at_index_mut(node.left) };
                    continue;
                }
                let index = self.add_node(key, value, hash);
                self.store[parent].left = index;

                return index;
            } else {
                if node.right != 0 {
                    parent = node.right;
                    node = unsafe { &mut *self.node_at_index_mut(node.right) };
                    continue;
                }
                let index = self.add_node(key, value, hash);
                self.store[parent].right = index;

                return index;
            }
        }
    }

    #[inline]
    pub(crate) fn override_at(&mut self, index: usize, value: JsonValue) {
        self.store[index].value = value;
    }

    #[inline]
    #[deprecated(since="0.11.11", note="Was only meant for internal use")]
    pub fn override_last(&mut self, value: JsonValue) {
        if let Some(node) = self.store.last_mut() {
            node.value = value;
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        if self.store.len() == 0 {
            return None;
        }

        let key = key.as_bytes();
        let hash = hash_key(key);

        let mut node = unsafe { self.store.get_unchecked(0) };

        loop {
            if hash == node.key.hash && key == node.key.as_bytes() {
                return Some(&node.value);
            } else if hash < node.key.hash {
                if node.left == 0 {
                    return None;
                }
                node = unsafe { self.store.get_unchecked(node.left) };
            } else {
                if node.right == 0 {
                    return None;
                }
                node = unsafe { self.store.get_unchecked(node.right) };
            }
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut JsonValue> {
        if self.store.len() == 0 {
            return None;
        }

        let key = key.as_bytes();
        let hash = hash_key(key);

        let mut index = 0;
        {
            let mut node = unsafe { self.store.get_unchecked(0) };

            loop {
                if hash == node.key.hash && key == node.key.as_bytes() {
                    break;
                } else if hash < node.key.hash {
                    if node.left == 0 {
                        return None;
                    }
                    index = node.left;
                    node = unsafe { self.store.get_unchecked(node.left) };
                } else {
                    if node.right == 0 {
                        return None;
                    }
                    index = node.right;
                    node = unsafe { self.store.get_unchecked(node.right) };
                }
            }
        }

        let node = unsafe { self.store.get_unchecked_mut(index) };

        Some(&mut node.value)
    }

    /// Attempts to remove the value behind `key`, if successful
    /// will return the `JsonValue` stored behind the `key`.
    pub fn remove(&mut self, key: &str) -> Option<JsonValue> {
        if self.store.len() == 0 {
            return None;
        }

        let key = key.as_bytes();
        let hash = hash_key(key);
        let mut index = 0;

        {
            let mut node = unsafe { self.store.get_unchecked(0) };

            // Try to find the node
            loop {
                if hash == node.key.hash && key == node.key.as_bytes() {
                    break;
                } else if hash < node.key.hash {
                    if node.left == 0 {
                        return None;
                    }
                    index = node.left;
                    node = unsafe { self.store.get_unchecked(node.left) };
                } else {
                    if node.right == 0 {
                        return None;
                    }
                    index = node.right;
                    node = unsafe { self.store.get_unchecked(node.right) };
                }
            }
        }

        // Removing a node would screw the tree badly, it's easier to just
        // recreate it. This is a very costly operation, but removing nodes
        // in JSON shouldn't happen very often if at all. Optimizing this
        // can wait for better times.
        let mut new_object = Object::with_capacity(self.store.len() - 1);
        let mut removed = None;

        for (i, node) in self.store.iter_mut().enumerate() {
            if i == index {
                // Rust doesn't like us moving things from `node`, even if
                // it is owned. Replace fixes that.
                removed = Some(mem::replace(&mut node.value, JsonValue::Null));
            } else {
                let value = mem::replace(&mut node.value, JsonValue::Null);

                new_object.insert(node.key.as_str(), value);
            }
        }

        mem::swap(self, &mut new_object);

        removed
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.store.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Wipe the `Object` clear. The capacity will remain untouched.
    pub fn clear(&mut self) {
        self.store.clear();
    }

    #[inline(always)]
    pub fn iter(&self) -> Iter {
        Iter {
            inner: self.store.iter()
        }
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            inner: self.store.iter_mut()
        }
    }

    /// Prints out the value as JSON string.
    pub fn dump(&self) -> String {
        let mut gen = DumpGenerator::new();
        gen.write_object(self).expect("Can't fail");
        gen.consume()
    }

    /// Pretty prints out the value as JSON string. Takes an argument that's
    /// number of spaces to indent new blocks with.
    pub fn pretty(&self, spaces: u16) -> String {
        let mut gen = PrettyGenerator::new(spaces);
        gen.write_object(self).expect("Can't fail");
        gen.consume()
    }
}

// Custom implementation of `Clone`, as new heap allocation means
// we have to fix key pointers everywhere!
impl Clone for Object {
    fn clone(&self) -> Self {
        let mut store = self.store.clone();

        for node in store.iter_mut() {
            node.key.fix_ptr();
        }

        Object {
            store: store
        }
    }
}

// Because keys can inserted in different order, the safe way to
// compare `Object`s is to iterate over one and check if the other
// has all the same keys.
impl PartialEq for Object {
    fn eq(&self, other: &Object) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for (key, value) in self.iter() {
            match other.get(key) {
                Some(ref other_val) => if *other_val != value { return false; },
                None                => return false
            }
        }

        true
    }
}

pub struct Iter<'a> {
    inner: slice::Iter<'a, Node>
}

impl<'a> Iter<'a> {
    /// Create an empty iterator that always returns `None`
    pub fn empty() -> Self {
        Iter {
            inner: [].iter()
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, &'a JsonValue);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|node| (node.key.as_str(), &node.value))
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|node| (node.key.as_str(), &node.value))
    }
}

pub struct IterMut<'a> {
    inner: slice::IterMut<'a, Node>
}

impl<'a> IterMut<'a> {
    /// Create an empty iterator that always returns `None`
    pub fn empty() -> Self {
        IterMut {
            inner: [].iter_mut()
        }
    }
}

impl<'a> Iterator for IterMut<'a> {
    type Item = (&'a str, &'a mut JsonValue);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|node| (node.key.as_str(), &mut node.value))
    }
}

impl<'a> DoubleEndedIterator for IterMut<'a> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|node| (node.key.as_str(), &mut node.value))
    }
}

/// Implements indexing by `&str` to easily access object members:
///
/// ## Example
///
/// ```
/// # #[macro_use]
/// # extern crate json;
/// # use json::JsonValue;
/// #
/// # fn main() {
/// let value = object!{
///     "foo" => "bar"
/// };
///
/// if let JsonValue::Object(object) = value {
///   assert!(object["foo"] == "bar");
/// }
/// # }
/// ```
// TODO: doc
impl<'a> Index<&'a str> for Object {
    type Output = JsonValue;

    fn index(&self, index: &str) -> &JsonValue {
        match self.get(index) {
            Some(value) => value,
            _ => &NULL
        }
    }
}

impl Index<String> for Object {
    type Output = JsonValue;

    fn index(&self, index: String) -> &JsonValue {
        self.index(index.deref())
    }
}

impl<'a> Index<&'a String> for Object {
    type Output = JsonValue;

    fn index(&self, index: &String) -> &JsonValue {
        self.index(index.deref())
    }
}

/// Implements mutable indexing by `&str` to easily modify object members:
///
/// ## Example
///
/// ```
/// # #[macro_use]
/// # extern crate json;
/// # use json::JsonValue;
/// #
/// # fn main() {
/// let value = object!{};
///
/// if let JsonValue::Object(mut object) = value {
///   object["foo"] = 42.into();
///
///   assert!(object["foo"] == 42);
/// }
/// # }
/// ```
impl<'a> IndexMut<&'a str> for Object {
    fn index_mut(&mut self, index: &str) -> &mut JsonValue {
        if self.get(index).is_none() {
            self.insert(index, JsonValue::Null);
        }
        self.get_mut(index).unwrap()
    }
}

impl IndexMut<String> for Object {
    fn index_mut(&mut self, index: String) -> &mut JsonValue {
        self.index_mut(index.deref())
    }
}

impl<'a> IndexMut<&'a String> for Object {
    fn index_mut(&mut self, index: &String) -> &mut JsonValue {
        self.index_mut(index.deref())
    }
}
