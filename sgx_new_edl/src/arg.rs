use crate::ecall::EcallArg;
use crate::ocall::OcallArg;
use crate::ser::*;

use std::boxed::Box;
use std::vec::Vec;

pub trait Update {
    fn update(&mut self, other: &Self);
}

pub struct In<'a, T: Encodable + Decodable> {
    inner: &'a T,
}

impl<'a, T: Decodable + Encodable> In<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Self { inner: value }
    }
}

impl<'a, T: Encodable + Decodable> EcallArg<T> for In<'a, T> {
    fn serialize(&self) -> Vec<u8> {
        // the address is all we need
        let ptr = self.inner as *const T as usize;
        serialize(&ptr).unwrap()
    }

    fn deserialize(data: &[u8]) -> Self {
        let addr: usize = deserialize(data).unwrap();
        let inner = unsafe { &*(addr as *mut T) };
        Self { inner }
    }

    fn prepare(&self) -> T {
        let bytes = serialize(self.inner).unwrap();
        deserialize(&bytes).unwrap()
    }

    unsafe fn _from_mut(ptr: &mut T) -> Self {
        Self {
            inner: &*(ptr as *mut T),
        }
    }

    fn update(&mut self, _: Self) {}
}

impl<'a, Target: Encodable + Decodable> OcallArg<Target> for In<'a, Target> {
    fn serialize(&self) -> Vec<u8> {
        let bytes = serialize(self.inner).unwrap();
        bytes
    }

    fn deserialize(data: &[u8]) -> Self {
        let inner = deserialize::<Target>(data).unwrap();
        Self {
            inner: Box::leak(Box::new(inner)),
        }
    }

    unsafe fn _clone(&mut self) -> Self {
        Self {
            inner: &*(self.inner as *const Target),
        }
    }
}

pub struct Out<'a, T: Decodable + Encodable + Update> {
    inner: &'a mut T,
}

impl<'a, T: Decodable + Encodable + Update> EcallArg<T> for Out<'a, T> {
    fn serialize(&self) -> Vec<u8> {
        // 我们需要记录位于enclave外部的指针，后续我们会使用
        let ptr = self.inner as *const T as usize;
        serialize(&ptr).unwrap()
    }

    fn deserialize(data: &[u8]) -> Self {
        let addr: usize = deserialize(data).unwrap();
        let inner = unsafe { &mut *(addr as *mut T) };
        Self { inner }
    }

    fn prepare(&self) -> T {
        let bytes = serialize(self.inner).unwrap();
        deserialize(&bytes).unwrap()
    }

    unsafe fn _from_mut(ptr: &mut T) -> Self {
        Self {
            inner: &mut *(ptr as *mut T),
        }
    }

    fn update(&mut self, other: Self) {
        self.inner.update(&other.inner);
    }
}

impl<'a, T: Update + Decodable + Encodable> OcallArg<T> for Out<'a, T> {
    fn serialize(&self) -> Vec<u8> {
        // 我们需要记录位于enclave外部的指针，后续我们会使用
        let ptr = self.inner as *const T as usize;
        serialize(&ptr).unwrap()
    }

    fn deserialize(data: &[u8]) -> Self {
        let addr: usize = deserialize(data).unwrap();
        let inner = unsafe { &mut *(addr as *mut T) };
        Self { inner }
    }

    unsafe fn _clone(&mut self) -> Self {
        Self {
            inner: &mut *(self.inner as *mut T),
        }
    }
}

impl<'a, T: Update + Decodable + Encodable> Out<'a, T> {
    pub fn new(value: &'a mut T) -> Self {
        Self { inner: value }
    }

    pub fn get(self) -> &'a T {
        self.inner
    }

    pub fn get_mut(mut self) -> &'a mut T {
        self.inner
    }
}

impl<T0, T1, A0, A1> EcallArg<(T0, T1)> for (A0, A1)
where
    A0: EcallArg<T0>,
    A1: EcallArg<T1>,
{
    fn serialize(&self) -> Vec<u8> {
        let value = (self.0.serialize(), self.1.serialize());
        serialize(&value).unwrap()
    }

    unsafe fn _from_mut(ptr: &mut (T0, T1)) -> Self {
        (A0::_from_mut(&mut ptr.0), A1::_from_mut(&mut ptr.1))
    }

    fn update(&mut self, other: (A0, A1)) {
        todo!()
    }

    fn prepare(&self) -> (T0, T1) {
        todo!()
    }

    fn deserialize(data: &[u8]) -> Self {
        let value = deserialize::<(Vec<u8>, Vec<u8>)>(data).unwrap();
        (A0::deserialize(&value.0), A1::deserialize(&value.1))
    }

    // unsafe fn _clone(&self) -> Self {
    //     (A0::_clone(&self.0), A1::_clone(&self.1))
    // }
}

// impl<T0, T1, A0, A1> OcallArg<(T0, T1)> for (A0, A1)
// where
//     A0: OcallArg<T0>,
//     A1: OcallArg<T1>,
// {
//     fn serialize(&self) -> Vec<u8> {
//         let value = (self.0.serialize(), self.1.serialize());
//         serialize(&value).unwrap()
//     }

//     fn deserialize(data: &[u8]) -> Self {
//         let value = deserialize::<(Vec<u8>, Vec<u8>)>(data).unwrap();
//         (A0::deserialize(&value.0), A1::deserialize(&value.1))
//     }

//     fn prepare(&self) -> (T0, T1) {
//         todo!()
//     }

//     unsafe fn _from_mut(target: &mut (T0, T1)) -> Self {
//         todo!()
//     }

//     fn update(&mut self, other: (T0, T1)) {
//         todo!()
//     }
// }

// pub trait OcallArg<Target> {
//     fn serialize(&self) -> Vec<u8>;
//     fn deserialize(data: &[u8]) -> Self;

//     fn prepare(&self) -> Target;

//     /// Reset lifetime
//     unsafe fn _from_mut(target: &mut Target) -> Self;

//     /// 将enclave内部的参数更新到外部
//     fn update(&mut self, other: Target);
// }

// impl<'a, Target: Encodable + Decodable> OcallArg<Target> for In<'a, Target> {
//     fn serialize(&self) -> Vec<u8> {
//         // 这里我们只需要内存地址
//         let ptr = self.inner as *const Target as usize;
//         serialize(&ptr).unwrap()
//     }

//     fn deserialize(data: &[u8]) -> Self {
//         let addr: usize = deserialize(data).unwrap();
//         let inner = unsafe { &*(addr as *mut Target) };
//         Self { inner }
//     }

//     fn prepare(&self) -> Target {
//         // 这里我们需要将内存中的数据反序列化为 Target 类型
//         let bytes = serialize(self.inner).unwrap();
//         deserialize(&bytes).unwrap()
//     }

//     unsafe fn _from_mut(target: &mut Target) -> Self {
//         Self {
//             inner: &*(target as *mut Target),
//         }
//     }

//     fn update(&mut self, other: Target) {
//         // 这里可以实现更新逻辑
//         // 例如，如果 Target 是一个可变引用，可以直接更新
//         // self.inner = other; // 具体实现取决于 Target 的类型
//     }
// }
