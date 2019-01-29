use std::prelude::v1::*;
use std::collections::HashMap;
use errors::VMError;
use value::Value;
use object::Object;
use object_info::{ObjectInfo, ObjectHandle, TypedObjectHandle};
use static_root::StaticRoot;
use call_stack::CallStack;
use errors;

/// An object pool that provides the backing object storage for executors.
pub struct ObjectPool {
    objects: Vec<Option<ObjectInfo>>,
    object_idx_pool: Vec<usize>,
    static_objects: HashMap<String, Value>,
    alloc_count: usize
}

impl ObjectPool {
    pub fn new() -> ObjectPool {
        ObjectPool {
            objects: vec![
                Some(ObjectInfo::new(Box::new(StaticRoot::new())))
            ],
            object_idx_pool: vec![],
            static_objects: HashMap::new(),
            alloc_count: 0
        }
    }

    /// Pins an object to the pool.
    pub fn allocate(&mut self, mut inner: Box<Object>) -> usize {
        inner.initialize(self);

        let id = if let Some(id) = self.object_idx_pool.pop() {
            id
        } else {
            let objects = &mut self.objects;
            objects.push(None);
            objects.len() - 1
        };
        self.objects[id] = Some(ObjectInfo::new(inner));

        self.alloc_count += 1;

        id
    }

    fn deallocate(&mut self, id: usize) {
        let objects = &mut self.objects;
        let pool = &mut self.object_idx_pool;

        assert!(objects[id].is_some());

        objects[id] = None;
        pool.push(id);
    }

    /// Gets a handle to the object at `id`.
    ///
    /// The handle can be passed around safely and
    /// the underlying object will not be garbage
    /// collected until all handles to it are released.
    ///
    /// If the object pool gets destroyed before
    /// all handles are dropped, the process will be
    /// aborted because of memory unsafety introduced
    /// by reference invalidation.
    pub fn get<'a>(&self, id: usize) -> ObjectHandle<'a> {
        self.objects[id].as_ref().unwrap().handle()
    }

    /// Gets a direct reference to the object at `id`.
    pub fn get_direct(&self, id: usize) -> &Object {
        self.objects[id].as_ref().unwrap().as_object()
    }

    /// Gets a direct typed reference to the object at `id`.
    /// If downcast fails, `None` is returned.
    pub fn get_direct_typed<T: 'static>(&self, id: usize) -> Option<&T> {
        self.get_direct(id).as_any().downcast_ref::<T>()
    }

    /// Gets a direct reference to the object at `id`.
    /// If downcast fails, this raises a `RuntimeError`.
    pub fn must_get_direct_typed<T: 'static>(&self, id: usize) -> &T {
        self.get_direct_typed(id).unwrap_or_else(|| {
            panic!(errors::VMError::from(errors::RuntimeError::new("Type mismatch")))
        })
    }

    /// Gets a typed object handle to the object at `id`.
    /// If downcast fails, `None` is returned.
    pub fn get_typed<'a, T: 'static>(&self, id: usize) -> Option<TypedObjectHandle<'a, T>> {
        TypedObjectHandle::downcast_from(self.get(id))
    }

    /// Gets a typed object handle to the object at `id`.
    /// If downcast fails, this raises a `RuntimeError`.
    pub fn must_get_typed<'a, T: 'static>(&self, id: usize) -> TypedObjectHandle<'a, T> {
        self.get_typed(id).unwrap_or_else(|| {
            panic!(errors::VMError::from(errors::RuntimeError::new("Type mismatch")))
        })
    }

    pub fn get_static_root<'a>(&self) -> TypedObjectHandle<'a, StaticRoot> {
        self.get_typed(0).unwrap()
    }

    pub fn get_direct_static_root(&self) -> &StaticRoot {
        self.get_direct_typed(0).unwrap()
    }

    pub fn set_static_object<K: ToString>(&mut self, key: K, obj: Value) {
        let key = key.to_string();

        // Replacing static objects is denied to ensure
        // `get_static_object_ref` is safe.
        if self.static_objects.get(key.as_str()).is_some() {
            panic!(VMError::from("A static object with the same key already exists"));
        }

        if let Value::Object(id) = obj {
            self.get_static_root().append_child(id);
        }
        self.static_objects.insert(key, obj);
    }

    pub fn get_static_object<K: AsRef<str>>(&self, key: K) -> Option<&Value> {
        let key = key.as_ref();
        self.static_objects.get(key)
    }

    pub fn get_alloc_count(&self) -> usize {
        self.alloc_count
    }

    pub fn reset_alloc_count(&mut self) {
        self.alloc_count = 0;
    }

    /// Run the garbage collector with the execution context
    /// provided by the given call stack.
    pub fn collect(&mut self, stack: &CallStack) {
        let mut visited: Vec<bool> = vec![false; self.objects.len()];

        let mut dfs: Vec<usize> = Vec::new();
        dfs.push(0); // static root

        for id in stack.collect_objects() {
            dfs.push(id);
        }

        while !dfs.is_empty() {
            let id = dfs.pop().unwrap();

            if visited[id] {
                continue;
            }
            visited[id] = true;

            let obj = &self.objects[id].as_ref().unwrap();
            for child in obj.as_object().get_children() {
                dfs.push(child);
            }
        }

        for i in 0..visited.len() {
            if self.objects[i].is_some() && !visited[i] {
                if !self.objects[i].as_ref().unwrap().has_native_refs() {
                    self.objects[i].as_mut().unwrap().gc_notify();
                    self.deallocate(i);
                }
            }
        }
    }
}

impl Drop for ObjectPool {
    fn drop(&mut self) {
        for obj in &mut self.objects {
            if let Some(ref mut obj) = *obj {
                obj.gc_notify();
            }
        }
    }
}
