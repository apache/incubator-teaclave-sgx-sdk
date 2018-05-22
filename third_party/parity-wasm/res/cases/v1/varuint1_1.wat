(module
  (type (;0;) (func (result i32)))
  (type (;1;) (func))
  (type (;2;) (func (param f32) (result f32)))
  (type (;3;) (func (param f64) (result f64)))
  (func (;0;) (type 0) (result i32)
    block  ;; label = @1
      get_global 0
      i32.eqz
      if  ;; label = @2
        i32.const 1
        return
      end
      get_global 0
      i32.const 1
      i32.sub
      set_global 0
    end
    block (result i32)  ;; label = @1
      nop
      loop (result i32)  ;; label = @2
        block  ;; label = @3
          get_global 0
          i32.eqz
          if  ;; label = @4
            i32.const 36
            return
          end
          get_global 0
          i32.const 1
          i32.sub
          set_global 0
        end
        i32.const 1684958791
        i32.const 1
        i32.const -32768
        if (result i32)  ;; label = @3
          loop (result i32)  ;; label = @4
            block  ;; label = @5
              get_global 0
              i32.eqz
              if  ;; label = @6
                i32.const -110
                return
              end
              get_global 0
              i32.const 1
              i32.sub
              set_global 0
            end
            i32.const -126
            loop (result i32)  ;; label = @5
              block  ;; label = @6
                get_global 0
                i32.eqz
                if  ;; label = @7
                  i32.const -12743
                  return
                end
                get_global 0
                i32.const 1
                i32.sub
                set_global 0
              end
              block (result i32)  ;; label = @6
                block  ;; label = @7
                  nop
                  block (result i32)  ;; label = @8
                    nop
                    br 1 (;@7;)
                  end
                  i32.const 15
                  i32.and
                  loop (result i32)  ;; label = @8
                    block  ;; label = @9
                      get_global 0
                      i32.eqz
                      if  ;; label = @10
                        i32.const 1
                        return
                      end
                      get_global 0
                      i32.const 1
                      i32.sub
                      set_global 0
                    end
                    block (result i32)  ;; label = @9
                      i32.const 36
                    end
                  end
                  i32.eqz
                  if (result i32)  ;; label = @8
                    i32.const -111
                  else
                    i32.const -3899777
                  end
                  i32.atomic.store16 offset=22
                end
                i32.const -29071
                i32.eqz
                br_if 1 (;@5;)
                i32.const -8388608
              end
            end
            br_if 3 (;@1;)
          end
        else
          i32.const -128
        end
        br_if 1 (;@1;)
        br_if 1 (;@1;)
      end
    end)
  (func (;1;) (type 1)
    i32.const 10
    set_global 0)
  (func (;2;) (type 2) (param f32) (result f32)
    get_local 0
    get_local 0
    f32.eq
    if (result f32)  ;; label = @1
      get_local 0
    else
      f32.const 0x0p+0 (;=0;)
    end)
  (func (;3;) (type 3) (param f64) (result f64)
    get_local 0
    get_local 0
    f64.eq
    if (result f64)  ;; label = @1
      get_local 0
    else
      f64.const 0x0p+0 (;=0;)
    end)
  (table (;0;) 0 0 anyfunc)
  (memory (;0;) 1 1 shared)
  (global (;0;) (mut i32) (i32.const 10))
  (export "func_0" (func 0))
  (export "hangLimitInitializer" (func 1))
  (elem (i32.const 0)))
