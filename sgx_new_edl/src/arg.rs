use serde::{Deserialize, Serialize};

pub trait Update {
    fn update(&mut self, other: &Self);
}

pub trait EcallArg<Target>: Sized {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Self;

    fn prepare(&self) -> Target;

    unsafe fn from_mut(target: &mut Target) -> Self;

    /// 将enclave内部的参数更新到外部
    fn update(&mut self, other: Target);

    fn destory(self);
}

pub struct In<'a, T: Serialize + for<'de> Deserialize<'de>> {
    inner: &'a T,
}

impl<'a, T: Serialize + for<'de> Deserialize<'de>> EcallArg<T> for In<'a, T> {
    fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self.inner).unwrap()
    }

    fn deserialize(data: &[u8]) -> Self {
        //bincode::deserialize(data).unwrap()
        todo!()
    }

    fn prepare(&self) -> T {
        todo!()
    }

    unsafe fn from_mut(ptr: &mut T) -> Self {
        Self {
            inner: &*(ptr as *mut T),
        }
    }

    fn update(&mut self, other: T) {}

    fn destory(self) {
        todo!()
    }
}

impl<'a, T: Serialize + for<'de> Deserialize<'de>> In<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Self { inner: value }
    }
}

pub struct Out<'a, T: Serialize + for<'de> Deserialize<'de> + Update> {
    inner: &'a mut T,
}

impl<'a, T: Serialize + for<'de> Deserialize<'de> + Update> EcallArg<T> for Out<'a, T> {
    fn serialize(&self) -> Vec<u8> {
        // 我们需要记录位于enclave外部的指针，后续我们会使用
        let ptr = self.inner as *const T as usize;
        bincode::serialize(&ptr).unwrap();
        todo!()
    }

    fn deserialize(data: &[u8]) -> Self {
        todo!()
    }

    fn prepare(&self) -> T {
        todo!()
    }

    unsafe fn from_mut(ptr: &mut T) -> Self {
        Self {
            inner: &mut *(ptr as *mut T),
        }
    }

    fn update(&mut self, other: T) {
        self.inner.update(&other);
    }

    fn destory(self) {
        todo!()
    }
}

impl<'a, T: Update + Serialize + for<'de> Deserialize<'de>> Out<'a, T> {
    pub fn new(value: &'a mut T) -> Self {
        Self { inner: value }
    }
}

impl<T0, T1, A0, A1> EcallArg<(T0, T1)> for (A0, A1)
where
    A0: EcallArg<T0>,
    A1: EcallArg<T1>,
{
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    unsafe fn from_mut(ptr: &mut (T0, T1)) -> Self {
        (A0::from_mut(&mut ptr.0), A1::from_mut(&mut ptr.1))
    }

    fn update(&mut self, other: (T0, T1)) {
        todo!()
    }

    fn destory(self) {
        todo!()
    }

    fn prepare(&self) -> (T0, T1) {
        todo!()
    }

    fn deserialize(data: &[u8]) -> Self {
        todo!()
    }
}
