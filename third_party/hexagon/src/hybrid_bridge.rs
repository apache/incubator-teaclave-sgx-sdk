// Obsolete.

use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use errors;
use hybrid::page_table::PageTable;
use object::Object;
use object_pool::ObjectPool;
use function::Function;
use executor::ExecutorImpl;
use value::{Value, ValueContext};

pub struct PageTableObject {
    pt: Rc<RefCell<PageTable>>,
    runtime_info: Option<PageTableRuntimeInfo>
}

struct PageTableRuntimeInfo {
    virtual_alloc_fn: usize
}

fn create_fn<T: Fn(&mut ExecutorImpl) -> Value + 'static>(pool: &mut ObjectPool, f: T) -> usize {
    pool.allocate(Box::new(Function::from_native(Box::new(f))))
}

impl Object for PageTableObject {
    fn initialize(&mut self, pool: &mut ObjectPool) {
        self.runtime_info = Some(PageTableRuntimeInfo {
            virtual_alloc_fn: create_fn(pool, {
                let pt = self.pt.clone();
                move |exec: &mut ExecutorImpl| {
                    let addr_p = exec.get_current_frame().must_get_argument(0);
                    let pool = exec.get_object_pool_mut();

                    let base = ValueContext::new(
                        &addr_p,
                        pool
                    ).to_i64();
                    let ok = pt.borrow_mut().virtual_alloc(base as u64);

                    if !ok {
                        panic!(errors::VMError::from(errors::RuntimeError::new("Virtual allocation failed")));
                    }

                    Value::Null
                }
            })
        });
    }

    fn get_children(&self) -> Vec<usize> {
        if let Some(ref rt_info) = self.runtime_info {
            vec! [
                rt_info.virtual_alloc_fn
            ]
        } else {
            Vec::new()
        }
    }

    fn as_any(&self) -> &Any {
        self as &Any
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self as &mut Any
    }
}

impl PageTableObject {
    pub fn new(pt: PageTable) -> PageTableObject {
        PageTableObject {
            pt: Rc::new(RefCell::new(pt)),
            runtime_info: None
        }
    }
}
