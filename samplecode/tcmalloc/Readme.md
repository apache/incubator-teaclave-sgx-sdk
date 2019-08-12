# tcmalloc code sample

This example shows how to use tcmalloc in rust-sgx enclaves.

This tcmalloc is provided by Intel and located at `${SGXSDK}/lib64/libsgx_tcmalloc.a`. To link against tcmalloc, the following link flag is required, and placed before link flag of `-lsgx_tstdc`:

```
-Wl,--whole-archive -lsgx_tcmalloc -Wl,--no-whole-archive
```

## One shot

```
$ make TCMALLOC=1
```

This would enable the linking flag `-lsgx_tcmalloc`.

```
$ make
```

This would use the default 'dlmalloc'.

## Comparison with traditional allocator (dlmalloc)

We provide a sample workload which only allocate buffers:

```rust
fn recursive_memory_func(x: u64, multiplier: u64) -> u64 {
    let v: Vec<u64> = Vec::with_capacity((x * multiplier) as usize);
    let p: u64 = v.as_ptr() as u64;
    //println!("ptr = {:X}", p);
    if x != 0 {
        p + recursive_memory_func(x - 1, multiplier)
    } else { p }
}
```

Small buffer test settings: `multiplier = 1` and initiate `x = 2000`.

Large buffer test settings: `multiplier = 100` and initiate `x = 1000`.

Please test the performance by yourself. Here is my result (i9-9900k, DDR4-3200):

|  | dlmalloc | tcmalloc |
| :-: | :-: | :-: |
| small payload (1x size, 2000 depth) | 1.306730572s | 674.858065ms |
| large payload (100x size, 1000 depth) | 874.595932ms | 1.519623607s |

