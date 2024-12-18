use bincode;
use serde::{Deserialize, Serialize};

use crate::ecall::{self, EcallArg};

pub trait Update {
    fn update(&mut self, other: &Self);
}

// pub struct Value<T: Serialize + for<'a> Deserialize<'a>> {
//     inner: T,
// }

// impl<'a, T: Serialize + for<'b> Deserialize<'b>> EcallArg for Value<T> {
//     fn serialize(&self) -> Vec<u8> {
//         bincode::serialize(&self.inner).unwrap()
//     }

//     fn deserialize(data: &[u8]) -> Self {
//         let t = bincode::deserialize::<T>(data).unwrap();
//         Self { inner: t }
//     }

//     fn update(&mut self, other: Self) {
//         other.destory();
//     }

//     fn prepare(&self) -> Result<Self, ecall::Error> {
//         todo!()
//     }

//     fn destory(self) {}
// }

pub struct In<'a, T: Serialize + for<'de> Deserialize<'de>> {
    inner: &'a T,
}

impl<'a, T: Serialize + for<'de> Deserialize<'de>> EcallArg for In<'a, T> {
    fn serialize(&self) -> Vec<u8> {
        let t = unsafe { &*self.inner };
        bincode::serialize(t).unwrap()
    }

    fn deserialize(data: &[u8]) -> Self {
        let t = bincode::deserialize::<T>(data).unwrap();
        Self {
            inner: Box::leak(Box::new(t)),
        }
    }

    fn update(&mut self, other: Self) {
        // In类型的参数不需要更新
        other.destory();
    }

    fn prepare(&self) -> Result<Self, ecall::Error> {
        // In类型的参数只需要拷贝一份引用
        Ok(Self { inner: self.inner })
    }

    fn destory(self) {
        // // 重新获取leak的所有权
        // let ptr = unsafe { Box::from_raw(self.inner as *const T as *mut T) };
        // drop(ptr);
    }
}

pub struct Out<'a, T: Serialize + for<'b> Deserialize<'b> + Update> {
    inner: &'a mut T,
}

impl<'a, T: Serialize + for<'b> Deserialize<'b> + Update> Out<'a, T> {
    pub fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { &*self.inner }
    }
}

impl<'a, T: Serialize + for<'b> Deserialize<'b> + Update> EcallArg for Out<'a, T> {
    fn serialize(&self) -> Vec<u8> {
        // 我们需要记录位于enclave外部的指针，后续我们会使用
        let ptr = self.inner as *const T as usize;
        bincode::serialize(&ptr).unwrap()
    }

    fn deserialize(data: &[u8]) -> Self {
        let ptr = bincode::deserialize::<usize>(data).unwrap();
        Self {
            inner: unsafe { &mut *(ptr as *mut T) },
        }
    }

    fn update(&mut self, other: Self) {
        self.as_mut().update(other.as_ref());
        // self.inner.update(other.as_mut());
        other.destory();
    }

    fn prepare(&self) -> Result<Self, ecall::Error> {
        // 创建一个新的Out类型的参数，这里重新序列化了一次，可以考虑增加Default约束
        let data = bincode::serialize(self.as_ref()).unwrap();
        let new = bincode::deserialize::<T>(&data).unwrap();
        Ok(Self {
            inner: Box::leak(Box::new(new)),
        })
    }

    fn destory(self) {
        let ptr = unsafe { Box::from_raw(self.inner as *const T as *mut T) };
        drop(ptr);
    }
}

pub struct InOut<'a, T: Serialize + for<'de> Deserialize<'de>> {
    inner: &'a mut T,
}

impl<'a, T: Serialize + for<'de> Deserialize<'de> + Update> EcallArg for InOut<'a, T> {
    fn serialize(&self) -> Vec<u8> {
        // 我们需要记录位于enclave外部的指针，后续我们会使用
        let ptr = self.inner as *const T as usize;
        bincode::serialize(&ptr).unwrap()
    }

    fn deserialize(data: &[u8]) -> Self {
        let ptr = bincode::deserialize::<usize>(data).unwrap();
        Self {
            inner: unsafe { &mut *(ptr as *mut T) },
        }
    }

    fn update(&mut self, other: Self) {
        self.inner.update(other.inner);
    }

    fn prepare(&self) -> Result<Self, ecall::Error> {
        let data = bincode::serialize(self.inner).unwrap();
        let new = bincode::deserialize::<T>(&data).unwrap();
        Ok(Self {
            inner: Box::leak(Box::new(new)),
        })
    }

    fn destory(self) {
        let ptr = unsafe { Box::from_raw(self.inner as *const T as *mut T) };
        drop(ptr);
    }
}

impl<A1: EcallArg, A2: EcallArg> EcallArg for (A1, A2) {
    fn serialize(&self) -> Vec<u8> {
        let a1 = self.0.serialize();
        let a2 = self.1.serialize();
        bincode::serialize(&(a1, a2)).unwrap()
    }

    fn deserialize(data: &[u8]) -> Self {
        let (d0, d1): (Vec<u8>, Vec<u8>) = bincode::deserialize(data).unwrap();
        (A1::deserialize(&d0), A2::deserialize(&d1))
    }

    fn prepare(&self) -> Result<Self, ecall::Error> {
        todo!()
    }

    fn update(&mut self, other: Self) {
        todo!()
    }

    fn destory(self) {
        todo!()
    }
}
