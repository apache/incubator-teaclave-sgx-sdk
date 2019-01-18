use std::prelude::v1::*;
use std::collections::{HashSet, BTreeSet};
use basic_block::BasicBlock;
use object_pool::ObjectPool;
use value::Value;

pub struct FunctionOptimizer<'a> {
    binded_this: Option<Value>,
    basic_blocks: &'a mut Vec<BasicBlock>,
    rt_handles: &'a mut Vec<usize>,
    pool: &'a mut ObjectPool
}

impl<'a> FunctionOptimizer<'a> {
    pub fn new(basic_blocks: &'a mut Vec<BasicBlock>, rt_handles: &'a mut Vec<usize>, pool: &'a mut ObjectPool) -> FunctionOptimizer<'a> {
        FunctionOptimizer {
            binded_this: None,
            basic_blocks: basic_blocks,
            rt_handles: rt_handles,
            pool: pool
        }
    }

    pub fn set_binded_this(&mut self, this: Option<Value>) {
        self.binded_this = this;
    }

    pub fn static_optimize(&mut self) {
        for _ in 0..3 {
            self.transform_const_locals();

            // Run optimizations on each basic block
            for bb in self.basic_blocks.iter_mut() {
                // LoadString -> LoadObject
                bb.transform_const_string_loads(self.rt_handles, self.pool);

                // (LoadObject, GetStatic) -> LoadValue
                bb.transform_const_static_loads(self.rt_handles, self.pool);

                while bb.transform_const_get_fields(self.rt_handles, self.pool, self.binded_this) {
                    bb.flatten_stack_maps();
                    bb.remove_nops();
                }
                bb.transform_const_calls();
                bb.remove_nops();
            }
        }

        // These should only run once
        for bb in self.basic_blocks.iter_mut() {
            bb.build_bulk_loads();
            bb.rebuild_stack_patterns();
        }

        self.simplify_cfg();
    }

    pub fn dynamic_optimize(&mut self) {
        for bb in self.basic_blocks.iter_mut() {
            // LoadString -> LoadObject
            bb.transform_const_string_loads(self.rt_handles, self.pool);

            // (LoadObject, GetStatic) -> LoadValue
            bb.transform_const_static_loads(self.rt_handles, self.pool);

            while bb.transform_const_get_fields(self.rt_handles, self.pool, self.binded_this) {
                bb.flatten_stack_maps();
                bb.remove_nops();
            }
            bb.transform_const_calls();
            bb.remove_nops();

            bb.build_bulk_loads();
        }
    }

    pub fn transform_const_locals(&mut self) {
        for bb in self.basic_blocks.iter_mut() {
            bb.transform_const_block_locals();
        }
    }

    pub fn simplify_cfg(&mut self) {
        if self.basic_blocks.len() == 0 {
            return;
        }

        let n_basic_blocks: usize = self.basic_blocks.len();

        let mut out_edges: Vec<HashSet<usize>> = vec! [ HashSet::new(); n_basic_blocks ];
        let mut in_edges: Vec<HashSet<usize>> = vec! [ HashSet::new(); n_basic_blocks ];
        for i in 0..n_basic_blocks {
            let (a, b) = self.basic_blocks[i].branch_targets();
            if let Some(v) = a {
                out_edges[i].insert(v);
                in_edges[v].insert(i);
            }
            if let Some(v) = b {
                out_edges[i].insert(v);
                in_edges[v].insert(i);
            }
        }

        for i in 0..n_basic_blocks {
            if out_edges[i].len() == 1 {
                let j = *out_edges[i].iter().nth(0).unwrap();
                if in_edges[j].len() == 1 {
                    if *in_edges[j].iter().nth(0).unwrap() == i {
                        println!("debug [simplify_cfg] Found unique connection: {} <-> {}", i, j);
                        out_edges.swap(i, j);
                        out_edges[j].clear();
                        in_edges[j].clear();
                        let v = ::std::mem::replace(
                            &mut self.basic_blocks[j],
                            BasicBlock::from_opcodes(Vec::new())
                        );
                        self.basic_blocks[i].join(v);
                    }
                }
            }
        }

        let mut dfs_stack: Vec<usize> = Vec::new();
        let mut dfs_visited: Vec<bool> = vec![ false; n_basic_blocks ];

        dfs_visited[0] = true;
        dfs_stack.push(0);

        while !dfs_stack.is_empty() {
            let current = dfs_stack.pop().unwrap();

            for other in &out_edges[current] {
                if !dfs_visited[*other] {
                    dfs_visited[*other] = true;
                    dfs_stack.push(*other);
                }
            }
        }

        // collect unused blocks
        {
            let unused_blocks: BTreeSet<usize> = (0..self.basic_blocks.len()).filter(|i| !dfs_visited[*i]).collect();
            let mut tail = n_basic_blocks - 1;
            let mut remap_list: Vec<(usize, usize)> = Vec::new(); // (to, from)
            for id in &unused_blocks {
                while tail > *id {
                    if unused_blocks.contains(&tail) {
                        tail -= 1;
                    } else {
                        break;
                    }
                }

                // Implies tail > 0
                if tail <= *id {
                    break;
                }

                // Now `id` is the first unused block and `tail`
                // is the last used block
                // Let's exchange them
                remap_list.push((*id, tail));
                self.basic_blocks.swap(*id, tail);
                tail -= 1;
            }
            while self.basic_blocks.len() > tail + 1 {
                self.basic_blocks.pop().unwrap();
            }
            for (to, from) in remap_list {
                for bb in self.basic_blocks.iter_mut() {
                    let replaced = bb.try_replace_branch_targets(to, from);
                    if replaced {
                        println!("debug [simplify_cfg] Branch target replaced: {} -> {}", from, to);
                    }
                }
            }
            //n_basic_blocks = self.basic_blocks.len();
        }
    }
}
