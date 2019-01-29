use std::any::Any;
use object::Object;
use object_pool::ObjectPool;
use basic_block::BasicBlock;
use opcode::{OpCode, RtOpCode, ValueLocation, StackMapPattern};
use value::Value;

#[test]
fn test_transform_const_calls() {
    let mut pool = ObjectPool::new();
    let obj_id = pool.allocate(Box::new("a".to_string()));
    //let mut rt_handles: Vec<usize> = Vec::new();

    let mut bb = BasicBlock::from_opcodes(vec! [
        { OpCode::LoadString("Hello world".into()) },
        { OpCode::LoadInt(42) },
        { OpCode::LoadNull },
        { OpCode::Rt(RtOpCode::LoadObject(obj_id)) },
        { OpCode::Call(1) },
        { OpCode::Pop },
        { OpCode::Return }
    ]);
    bb.transform_const_calls();
    bb.remove_nops();

    assert_eq!(bb.opcodes, vec! [
        { OpCode::LoadString("Hello world".into()) },
        { OpCode::LoadInt(42) },
        { OpCode::Rt(RtOpCode::ConstCall(
            ValueLocation::ConstObject(obj_id),
            ValueLocation::ConstNull,
            1
        )) },
        { OpCode::Pop },
        { OpCode::Return }
    ]);
}

struct TestObject {

}

impl Object for TestObject {
    fn get_children(&self) -> Vec<usize> {
        Vec::new()
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }

    fn get_field(&self, _: &ObjectPool, name: &str) -> Option<Value> {
        if name == "key" {
            Some(Value::Bool(true))
        } else {
            None
        }
    }

    fn has_const_field(&self, _: &ObjectPool, name: &str) -> bool {
        if name == "key" {
            true
        } else {
            false
        }
    }
}

#[test]
fn test_transform_const_get_fields() {
    let mut pool = ObjectPool::new();
    let mut rt_handles: Vec<usize> = Vec::new();
    let this_id = pool.allocate(Box::new(TestObject {}));
    let key_id = pool.allocate(Box::new("key".to_string()));

    let mut bb = BasicBlock::from_opcodes(vec! [
        { OpCode::LoadNull },
        { OpCode::LoadInt(1) },
        { OpCode::Pop },
        { OpCode::Pop },
        { OpCode::Rt(RtOpCode::LoadObject(key_id)) },
        { OpCode::Rt(RtOpCode::LoadObject(this_id)) },
        { OpCode::GetField },
        { OpCode::Pop },
        { OpCode::Rt(RtOpCode::LoadObject(key_id)) },
        { OpCode::LoadNull },
        { OpCode::Rt(RtOpCode::LoadObject(this_id)) },
        { OpCode::CallField(0) },
        { OpCode::Return }
    ]);
    while bb.transform_const_get_fields(&mut rt_handles, &mut pool, Some(Value::Object(this_id))) {
        bb.remove_nops();
    }

    assert_eq!(bb.opcodes, vec! [
        { OpCode::Rt(RtOpCode::StackMap(StackMapPattern {
            map: (&[] as &[ValueLocation]).into(),
            end_state: 0
        }))},
        { OpCode::Rt(RtOpCode::StackMap(StackMapPattern {
            map: (&[
                ValueLocation::ConstNull
            ] as &[ValueLocation]).into(),
            end_state: 1
        }))},
        { OpCode::LoadBool(true) },
        { OpCode::Call(0) },
        { OpCode::Return }
    ]);
}
