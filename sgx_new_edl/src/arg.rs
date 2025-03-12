use crate::ecall::EcallArg;
use crate::ocall::OcallArg;
use crate::ser::*;

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

    fn update(&mut self, _: T) {}
}

impl<'a, Target: Encodable + Decodable> OcallArg<Target> for In<'a, Target> {
    fn serialize(&self) -> Vec<u8> {
        todo!()
    }

    fn deserialize(data: &[u8]) -> Self {
        todo!()
    }

    fn prepare(&self) -> Target {
        todo!()
    }

    unsafe fn _from_mut(target: &mut Target) -> Self {
        todo!()
    }

    fn update(&mut self, other: Target) {
        todo!()
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

    fn update(&mut self, other: T) {
        self.inner.update(&other);
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

    fn prepare(&self) -> T {
        // 这里我们需要将内存中的数据反序列化为 T 类型
        let bytes = serialize(self.inner).unwrap();
        deserialize(&bytes).unwrap()
    }

    unsafe fn _from_mut(ptr: &mut T) -> Self {
        Self {
            inner: &mut *(ptr as *mut T),
        }
    }

    fn update(&mut self, other: T) {
        self.inner.update(&other);
    }
}

impl<'a, T: Update + Decodable + Encodable> Out<'a, T> {
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
        let value = (self.0.serialize(), self.1.serialize());
        serialize(&value).unwrap()
    }

    unsafe fn _from_mut(ptr: &mut (T0, T1)) -> Self {
        (A0::_from_mut(&mut ptr.0), A1::_from_mut(&mut ptr.1))
    }

    fn update(&mut self, other: (T0, T1)) {
        todo!()
    }

    fn prepare(&self) -> (T0, T1) {
        todo!()
    }

    fn deserialize(data: &[u8]) -> Self {
        todo!()
    }
}

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
