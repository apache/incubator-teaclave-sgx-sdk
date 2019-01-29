use std::cell::{Cell, RefCell, Ref};
use super::page_table::PageTable;
use super::function::Function;
use super::opcode::OpCode;
use super::type_cast;
use super::program::Program;
use super::program_context::{ProgramContext, CommonProgramContext};
use super::jit::NoJit;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

pub struct Executor {
    page_table: RefCell<PageTable>,
    globals: [Cell<u64>; 16],
    call_stack_depth: Cell<usize>,
    max_call_stack_depth: usize
}

struct Local {
    regs: [u64; 16]
}

enum EvalControlMessage {
    Return,
    Redirect(usize)
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            page_table: RefCell::new(PageTable::new()),
            globals: build_global_regs(),
            call_stack_depth: Cell::new(0),
            max_call_stack_depth: 512
        }
    }

    pub fn with_page_table(pt: PageTable) -> Executor {
        Executor {
            page_table: RefCell::new(pt),
            globals: build_global_regs(),
            call_stack_depth: Cell::new(0),
            max_call_stack_depth: 512
        }
    }

    pub fn get_page_table<'a>(&'a self) -> Ref<'a, PageTable> {
        self.page_table.borrow()
    }

    pub fn set_page_table(&self, pt: PageTable) {
        *self.page_table.borrow_mut() = pt;
    }

    pub fn read_global(&self, id: usize) -> u64 {
        self.globals[id].get()
    }

    pub fn write_global(&self, id: usize, value: u64) {
        self.globals[id].replace(value);
    }

    fn eval_partial(
        &self,
        program: &CommonProgramContext,
        local: &mut Local,
        f: &Function,
        block_id: usize
    ) -> EvalControlMessage {
        let blk = &f.basic_blocks[block_id];
        for op in blk.opcodes.iter() {
            match *op {
                OpCode::Return => {
                    return EvalControlMessage::Return;
                },
                OpCode::Branch(target) => {
                    return EvalControlMessage::Redirect(target);
                },
                OpCode::ConditionalBranch(a, b) => {
                    return EvalControlMessage::Redirect(if local.regs[0] != 0 {
                        a
                    } else {
                        b
                    });
                },
                OpCode::SIAdd(a, b) => {
                    local.regs[0] = (local.regs[a] as i64 + local.regs[b] as i64) as u64;
                },
                OpCode::SISub(a, b) => {
                    local.regs[0] = (local.regs[a] as i64 - local.regs[b] as i64) as u64;
                },
                OpCode::SIMul(a, b) => {
                    local.regs[0] = (local.regs[a] as i64 * local.regs[b] as i64) as u64;
                },
                OpCode::SIDiv(a, b) => {
                    local.regs[0] = (local.regs[a] as i64 / local.regs[b] as i64) as u64;
                },
                OpCode::SIMod(a, b) => {
                    local.regs[0] = (local.regs[a] as i64 % local.regs[b] as i64) as u64;
                },
                OpCode::UIAdd(a, b) => {
                    local.regs[0] = (local.regs[a] as u64 + local.regs[b] as u64) as u64;
                },
                OpCode::UISub(a, b) => {
                    local.regs[0] = (local.regs[a] as u64 - local.regs[b] as u64) as u64;
                },
                OpCode::UIMul(a, b) => {
                    local.regs[0] = (local.regs[a] as u64 * local.regs[b] as u64) as u64;
                },
                OpCode::UIDiv(a, b) => {
                    local.regs[0] = (local.regs[a] as u64 / local.regs[b] as u64) as u64;
                },
                OpCode::UIMod(a, b) => {
                    local.regs[0] = (local.regs[a] as u64 % local.regs[b] as u64) as u64;
                },
                OpCode::FAdd(a, b) => {
                    local.regs[0] = type_cast::f64_to_u64(
                        type_cast::u64_to_f64(local.regs[a]).unwrap() +
                        type_cast::u64_to_f64(local.regs[b]).unwrap()
                    );
                },
                OpCode::FSub(a, b) => {
                    local.regs[0] = type_cast::f64_to_u64(
                        type_cast::u64_to_f64(local.regs[a]).unwrap() -
                        type_cast::u64_to_f64(local.regs[b]).unwrap()
                    );
                },
                OpCode::FMul(a, b) => {
                    local.regs[0] = type_cast::f64_to_u64(
                        type_cast::u64_to_f64(local.regs[a]).unwrap() *
                        type_cast::u64_to_f64(local.regs[b]).unwrap()
                    );
                },
                OpCode::FDiv(a, b) => {
                    local.regs[0] = type_cast::f64_to_u64(
                        type_cast::u64_to_f64(local.regs[a]).unwrap() /
                        type_cast::u64_to_f64(local.regs[b]).unwrap()
                    );
                },
                OpCode::FMod(a, b) => {
                    local.regs[0] = type_cast::f64_to_u64(
                        type_cast::u64_to_f64(local.regs[a]).unwrap() %
                        type_cast::u64_to_f64(local.regs[b]).unwrap()
                    );
                },
                OpCode::Shl(a, b) => {
                    local.regs[0] = local.regs[a] << local.regs[b];
                },
                OpCode::Shr(a, b) => {
                    local.regs[0] = local.regs[a] >> local.regs[b];
                },
                OpCode::BitAnd(a, b) => {
                    local.regs[0] = local.regs[a] & local.regs[b];
                },
                OpCode::BitOr(a, b) => {
                    local.regs[0] = local.regs[a] | local.regs[b];
                },
                OpCode::Xor(a, b) => {
                    local.regs[0] = local.regs[a] ^ local.regs[b];
                },
                OpCode::LogicalNot(v) => {
                    local.regs[0] = if local.regs[v] != 0 {
                        1
                    } else {
                        0
                    };
                },
                OpCode::BitNot(v) => {
                    local.regs[0] = !local.regs[v];
                },
                OpCode::SILt(a, b) => {
                    local.regs[0] = if (local.regs[a] as i64) < (local.regs[b] as i64) {
                        1
                    } else {
                        0
                    };
                },
                OpCode::SILe(a, b) => {
                    local.regs[0] = if local.regs[a] as i64 <= local.regs[b] as i64 {
                        1
                    } else {
                        0
                    };
                },
                OpCode::SIGe(a, b) => {
                    local.regs[0] = if local.regs[a] as i64 >= local.regs[b] as i64 {
                        1
                    } else {
                        0
                    };
                },
                OpCode::SIGt(a, b) => {
                    local.regs[0] = if local.regs[a] as i64 > local.regs[b] as i64 {
                        1
                    } else {
                        0
                    };
                },
                OpCode::UILt(a, b) => {
                    local.regs[0] = if (local.regs[a] as u64) < (local.regs[b] as u64) {
                        1
                    } else {
                        0
                    };
                },
                OpCode::UILe(a, b) => {
                    local.regs[0] = if local.regs[a] as u64 <= local.regs[b] as u64 {
                        1
                    } else {
                        0
                    };
                },
                OpCode::UIGe(a, b) => {
                    local.regs[0] = if local.regs[a] as u64 >= local.regs[b] as u64 {
                        1
                    } else {
                        0
                    };
                },
                OpCode::UIGt(a, b) => {
                    local.regs[0] = if local.regs[a] as u64 > local.regs[b] as u64 {
                        1
                    } else {
                        0
                    };
                },
                OpCode::FLt(a, b) => {
                    let (left, right) = (
                        type_cast::u64_to_f64(local.regs[a]).unwrap(),
                        type_cast::u64_to_f64(local.regs[b]).unwrap()
                    );
                    local.regs[0] = if left < right {
                        1
                    } else {
                        0
                    };
                },
                OpCode::FLe(a, b) => {
                    let (left, right) = (
                        type_cast::u64_to_f64(local.regs[a]).unwrap(),
                        type_cast::u64_to_f64(local.regs[b]).unwrap()
                    );
                    local.regs[0] = if left <= right {
                        1
                    } else {
                        0
                    };
                },
                OpCode::FGe(a, b) => {
                    let (left, right) = (
                        type_cast::u64_to_f64(local.regs[a]).unwrap(),
                        type_cast::u64_to_f64(local.regs[b]).unwrap()
                    );
                    local.regs[0] = if left >= right {
                        1
                    } else {
                        0
                    };
                },
                OpCode::FGt(a, b) => {
                    let (left, right) = (
                        type_cast::u64_to_f64(local.regs[a]).unwrap(),
                        type_cast::u64_to_f64(local.regs[b]).unwrap()
                    );
                    local.regs[0] = if left > right {
                        1
                    } else {
                        0
                    };
                },
                OpCode::Eq(a, b) => {
                    local.regs[0] = if local.regs[a] == local.regs[b] {
                        1
                    } else {
                        0
                    };
                },
                OpCode::Ne(a, b) => {
                    local.regs[0] = if local.regs[a] != local.regs[b] {
                        1
                    } else {
                        0
                    };
                },
                OpCode::SIConst8(target, v) => {
                    local.regs[target] = v as u64;
                },
                OpCode::SIConst16(target, v) => {
                    local.regs[target] = v as u64;
                },
                OpCode::SIConst32(target, v) => {
                    local.regs[target] = v as u64;
                },
                OpCode::SIConst64(target, v) => {
                    local.regs[target] = v as u64;
                },
                OpCode::UIConst8(target, v) => {
                    local.regs[target] = v as u64;
                },
                OpCode::UIConst16(target, v) => {
                    local.regs[target] = v as u64;
                },
                OpCode::UIConst32(target, v) => {
                    local.regs[target] = v as u64;
                },
                OpCode::UIConst64(target, v) => {
                    local.regs[target] = v as u64;
                },
                OpCode::FConst64(target, v) => {
                    local.regs[target] = type_cast::f64_to_u64(v);
                },
                OpCode::Load8(target, p) => {
                    let addr = local.regs[p];
                    local.regs[target] = self.page_table.borrow_mut().read_u8(addr).unwrap() as u64;
                },
                OpCode::Load16(target, p) => {
                    let addr = local.regs[p];
                    local.regs[target] = self.page_table.borrow_mut().read_u16(addr).unwrap() as u64;
                },
                OpCode::Load32(target, p) => {
                    let addr = local.regs[p];
                    local.regs[target] = self.page_table.borrow_mut().read_u32(addr).unwrap() as u64;
                },
                OpCode::Load64(target, p) => {
                    let addr = local.regs[p];
                    local.regs[target] = self.page_table.borrow_mut().read_u64(addr).unwrap() as u64;
                },
                OpCode::Store8(src, p) => {
                    let addr = local.regs[p];
                    self.page_table.borrow_mut().write_u8(addr, (local.regs[src] & 0xff) as u8);
                },
                OpCode::Store16(src, p) => {
                    let addr = local.regs[p];
                    self.page_table.borrow_mut().write_u16(addr, (local.regs[src] & 0xffff) as u16);
                },
                OpCode::Store32(src, p) => {
                    let addr = local.regs[p];
                    self.page_table.borrow_mut().write_u32(addr, (local.regs[src] & 0xffffffff) as u32);
                },
                OpCode::Store64(src, p) => {
                    let addr = local.regs[p];
                    self.page_table.borrow_mut().write_u64(addr, local.regs[src]);
                },
                OpCode::Mov(dst, src) => {
                    local.regs[dst] = local.regs[src];
                },
                OpCode::LoadGlobal(dst, src) => {
                    local.regs[dst] = self.read_global(src);
                },
                OpCode::StoreGlobal(dst, src) => {
                    self.write_global(dst, local.regs[src]);
                },
                OpCode::Call(target) => {
                    self.eval_program(program, target);
                },
                OpCode::CallIndirect(target) => {
                    let target = local.regs[target] as usize;
                    self.eval_program(program, target);
                },
                OpCode::CallNative(target) => {
                    program.get_program().native_functions[target].invoke(self);
                },
                OpCode::CallNativeIndirect(target) => {
                    let target = local.regs[target] as usize;
                    program.get_program().native_functions[target].invoke(self);
                }
            }
        }
        panic!("Terminator not found");
    }

    pub fn eval_program(&self, program: &CommonProgramContext, entry_fn: usize) {

        let entry = &program.get_program().functions[entry_fn];

        if self.call_stack_depth.get() >= self.max_call_stack_depth {
            panic!("Max call stack depth exceeded");
        }

        // FIXME: It seems that the Cell has a negative impact on performance
        // (bench_invoke: 20ns -> 28ns)
        self.call_stack_depth.replace(self.call_stack_depth.get() + 1);

        let result = catch_unwind(AssertUnwindSafe(|| {
            if let Some(provider) = program.get_jit_provider() {
                if provider.invoke_function(program, entry_fn) == true {
                    return;
                }
            }

            let mut local = Local {
                regs: [0u64; 16]
            };
            let mut block_id: usize = 0;

            loop {
                match self.eval_partial(program, &mut local, entry, block_id) {
                    EvalControlMessage::Return => {
                        break;
                    },
                    EvalControlMessage::Redirect(target) => {
                        block_id = target;
                    }
                }
            }
        }));
        
        self.call_stack_depth.replace(self.call_stack_depth.get() - 1);

        if let Err(e) = result {
            resume_unwind(e);
        }
    }

    pub fn eval_function(&self, f: &Function) {
        let program = Program::from_functions(vec! [ f.clone() ]);
        let ctx = ProgramContext::new(self, program, None as Option<NoJit>);
        self.eval_program(&ctx, 0)
    }
}

fn build_global_regs() -> [Cell<u64>; 16] {
    [
        Cell::new(0),
        Cell::new(0),
        Cell::new(0),
        Cell::new(0),
        Cell::new(0),
        Cell::new(0),
        Cell::new(0),
        Cell::new(0),
        Cell::new(0),
        Cell::new(0),
        Cell::new(0),
        Cell::new(0),
        Cell::new(0),
        Cell::new(0),
        Cell::new(0),
        Cell::new(0)
    ]
}
