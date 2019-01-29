use std::prelude::v1::*;
use smallvec::SmallVec;
use call_stack::Frame;
use object_pool::ObjectPool;
use value::Value;
use errors::ValidateError;

/// Hexagon VM opcodes.
///
/// Note that the `Rt` variant is only meant to be used internally
/// by the optimizer and will not pass code validation at function
/// creation.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum OpCode {
    Nop,
    LoadNull,
    LoadInt(i64),
    LoadFloat(f64),
    LoadString(String),
    LoadBool(bool),
    LoadThis,
    Pop,
    Dup,
    InitLocal(usize),
    GetLocal(usize),
    SetLocal(usize),
    GetArgument(usize),
    GetNArguments,
    GetStatic,
    SetStatic,
    GetField,
    SetField,
    Call(usize),
    CallField(usize),
    Branch(usize),
    ConditionalBranch(usize, usize),
    Return,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    IntAdd,
    IntSub,
    IntMul,
    IntDiv,
    IntMod,
    IntPow,
    FloatAdd,
    FloatSub,
    FloatMul,
    FloatDiv,
    FloatPowi,
    FloatPowf,
    StringAdd,
    CastToFloat,
    CastToInt,
    CastToBool,
    CastToString,
    And,
    Or,
    Not,
    TestLt,
    TestLe,
    TestEq,
    TestNe,
    TestGe,
    TestGt,
    Rotate2,
    Rotate3,
    RotateReverse(usize),

    // used for short-circuiting operations
    // both blocks must pop no value and produce exactly one value
    Select(SelectType, Vec<OpCode>, Vec<OpCode>),

    #[serde(skip_serializing, skip_deserializing)]
    Rt(RtOpCode)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SelectType {
    And,
    Or
}

#[derive(Clone, Debug, PartialEq)]
pub enum RtOpCode {
    LoadObject(usize),
    BulkLoad(SmallVec<[Value; 4]>),
    StackMap(StackMapPattern),
    ConstCall(ValueLocation /* target */, ValueLocation /* this */, usize /* n_args */),
    ConstGetField(usize /* object id */, Value /* key */)
}

#[derive(Clone, Debug, PartialEq)]
pub struct StackMapPattern {
    pub(crate) map: SmallVec<[ValueLocation; 4]>,
    pub(crate) end_state: isize
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueLocation {
    Stack(isize),
    Local(usize),
    Argument(usize),
    ConstInt(i64),
    ConstFloat(f64),
    ConstString(String),
    ConstBool(bool),
    ConstNull,
    ConstObject(usize),
    This
}

impl StackMapPattern {
    pub fn to_opcode_sequence(&self) -> Option<Vec<OpCode>> {
        let mut opcodes: Vec<OpCode> = Vec::new();
        let n_pushes: isize = self.end_state - self.map.len() as isize;
        if n_pushes > 0 {
            return None;
        } else {
            for _ in 0..(-n_pushes) {
                opcodes.push(OpCode::Pop);
            }
        }

        for loc in self.map.iter() {
            if let Some(op) = loc.to_opcode() {
                opcodes.push(op);
            } else {
                return None;
            }
        }
        Some(opcodes)
    }
}

impl ValueLocation {
    pub fn to_opcode(&self) -> Option<OpCode> {
        match *self {
            ValueLocation::Stack(_) => None,
            ValueLocation::Local(id) => Some(OpCode::GetLocal(id)),
            ValueLocation::Argument(id) => Some(OpCode::GetArgument(id)),
            ValueLocation::ConstInt(v) => Some(OpCode::LoadInt(v)),
            ValueLocation::ConstFloat(v) => Some(OpCode::LoadFloat(v)),
            ValueLocation::ConstString(ref s) => Some(OpCode::LoadString(s.clone())),
            ValueLocation::ConstBool(v) => Some(OpCode::LoadBool(v)),
            ValueLocation::ConstNull => Some(OpCode::LoadNull),
            ValueLocation::ConstObject(id) => Some(OpCode::Rt(RtOpCode::LoadObject(id))),
            ValueLocation::This => Some(OpCode::LoadThis)
        }
    }

    pub fn from_opcode(op: &OpCode) -> Option<ValueLocation> {
        match *op {
            OpCode::GetLocal(id) => Some(ValueLocation::Local(id)),
            OpCode::GetArgument(id) => Some(ValueLocation::Argument(id)),
            OpCode::LoadInt(v) => Some(ValueLocation::ConstInt(v)),
            OpCode::LoadFloat(v) => Some(ValueLocation::ConstFloat(v)),
            OpCode::LoadString(ref v) => Some(ValueLocation::ConstString(v.clone())),
            OpCode::LoadBool(v) => Some(ValueLocation::ConstBool(v)),
            OpCode::LoadNull => Some(ValueLocation::ConstNull),
            OpCode::Rt(RtOpCode::LoadObject(id)) => Some(ValueLocation::ConstObject(id)),
            OpCode::LoadThis => Some(ValueLocation::This),
            _ => None
        }
    }

    pub fn extract(&self, frame: &Frame, pool: &mut ObjectPool) -> Value {
        match *self {
            ValueLocation::Stack(dt) => {
                let center = frame.exec_stack.len() - 1;
                frame.exec_stack.get((center as isize + dt) as usize).unwrap()
            },
            ValueLocation::Local(id) => frame.get_local(id),
            ValueLocation::Argument(id) => frame.must_get_argument(id),
            ValueLocation::ConstString(ref s) => {
                Value::Object(pool.allocate(Box::new(s.clone())))
            },
            ValueLocation::ConstNull => Value::Null,
            ValueLocation::ConstInt(v) => Value::Int(v),
            ValueLocation::ConstFloat(v) => Value::Float(v),
            ValueLocation::ConstBool(v) => Value::Bool(v),
            ValueLocation::ConstObject(id) => Value::Object(id),
            ValueLocation::This => frame.get_this()
        }
    }

    pub fn to_value(&self) -> Option<Value> {
        match *self {
            ValueLocation::ConstNull => Some(Value::Null),
            ValueLocation::ConstInt(v) => Some(Value::Int(v)),
            ValueLocation::ConstFloat(v) => Some(Value::Float(v)),
            ValueLocation::ConstBool(v) => Some(Value::Bool(v)),
            ValueLocation::ConstObject(v) => Some(Value::Object(v)),
            _ => None
        }
    }
}

macro_rules! validate_select_opcode_sequence {
    ($seq:expr) => ({
        {
            let mut stack_depth: isize = 0;
            for op in $seq {
                op.validate(false)?;
                let (n_pops, n_pushes) = op.get_stack_depth_change();
                stack_depth -= n_pops as isize;
                if stack_depth < 0 {
                    return Err(ValidateError::new("Stack underflow"));
                }
                stack_depth += n_pushes as isize;
            }
            if stack_depth != 1 {
                return Err(ValidateError::new("Expecting exactly one value"));
            }
        }
    })
}

impl OpCode {
    pub fn modifies_control_flow(&self) -> bool {
        match *self {
            OpCode::Branch(_) | OpCode::ConditionalBranch(_, _) | OpCode::Return => true,
            _ => false
        }
    }

    pub fn validate(&self, allow_modify_control_flow: bool) -> Result<(), ValidateError> {
        if !allow_modify_control_flow {
            if self.modifies_control_flow() {
                return Err(ValidateError::new("Modifying control flow is not allowed here"));
            }
        }

        match *self {
            OpCode::RotateReverse(n) => {
                if n <= 0 {
                    return Err(ValidateError::new("RotateReverse only accepts an operand greater than zero"));
                }
                Ok(())
            },
            OpCode::Select(_, ref left, ref right) => {
                validate_select_opcode_sequence!(left);
                validate_select_opcode_sequence!(right);
                Ok(())
            },
            _ => Ok(())
        }
    }

    pub fn from_value(v: Value) -> OpCode {
        use self::OpCode::*;

        match v {
            Value::Null => LoadNull,
            Value::Bool(v) => LoadBool(v),
            Value::Int(v) => LoadInt(v),
            Value::Float(v) => LoadFloat(v),
            Value::Object(v) => Rt(RtOpCode::LoadObject(v))
        }
    }

    pub fn to_value(&self) -> Option<Value> {
        use self::OpCode::*;

        match *self {
            LoadNull => Some(Value::Null),
            LoadBool(v) => Some(Value::Bool(v)),
            LoadInt(v) => Some(Value::Int(v)),
            LoadFloat(v) => Some(Value::Float(v)),
            Rt(RtOpCode::LoadObject(id)) => Some(Value::Object(id)),
            _ => None
        }
    }

    pub fn get_stack_depth_change(&self) -> (usize, usize) {
        use self::OpCode::*;

        // (pop, push)
        match *self {
            Nop => (0, 0),
            LoadNull | LoadInt(_) | LoadFloat(_) | LoadString(_) | LoadBool(_) | LoadThis => (0, 1), // pushes the value
            Pop => (1, 0), // pops the object on the top
            Dup => (0, 1), // duplicates the object on the top
            InitLocal(_) => (0, 0),
            GetLocal(_) => (0, 1), // pushes object
            SetLocal(_) => (1, 0), // pops object
            GetArgument(_) => (0, 1), // pushes the argument
            GetNArguments => (0, 1), // pushes n_arguments
            GetStatic => (1, 1), // pops name, pushes object
            SetStatic => (2, 0), // pops name & object
            GetField => (2, 1), // pops target object & key, pushes object
            SetField => (3, 0), // pops target object & key & value
            Branch(_) => (0, 0),
            ConditionalBranch(_, _) => (1, 0), // pops condition
            Return => (1, 0), // pops retval,
            Add | Sub | Mul | Div | Mod | Pow
                | IntAdd | IntSub | IntMul | IntDiv | IntMod | IntPow
                | FloatAdd | FloatSub | FloatMul | FloatDiv
                | FloatPowi | FloatPowf
                | StringAdd => (2, 1), // pops the two operands, pushes the result
            CastToFloat | CastToInt | CastToBool | CastToString => (1, 1),
            Not => (1, 1),
            And | Or => (2, 1), // pops the two operands, pushes the result
            TestLt | TestLe | TestEq | TestNe | TestGe | TestGt => (2, 1), // pops the two operands, pushes the result
            Call(n_args) => (n_args + 2, 1), // pops target & this & arguments, pushes the result
            CallField(n_args) => (n_args + 3, 1), // pops target & this & field_name & arguments, pushes the result
            Rotate2 => (2, 2),
            Rotate3 => (3, 3),
            RotateReverse(n) => (n, n),
            Select(_, _, _) => (0, 1), // pushes exactly one value
            Rt(ref op) => match *op {
                RtOpCode::LoadObject(_) => (0, 1), // pushes the object at id
                RtOpCode::BulkLoad(ref values) => (0, values.len()), // pushes all the values
                RtOpCode::StackMap(ref p) => if p.end_state >= 0 {
                    (0, p.end_state as usize)
                } else {
                    ((-p.end_state) as usize, 0)
                },
                RtOpCode::ConstCall(_, _, n_args) => (n_args, 1), // pops arguments, pushes the result
                RtOpCode::ConstGetField(_, _) => (0, 1) // pushes the object
            }
        }
    }
}
