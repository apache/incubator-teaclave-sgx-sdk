(module
  (type    (func))                           ;; 0: void -> void
  (type $S (func))                           ;; 1: void -> void
  (type    (func (param)))                   ;; 2: void -> void
  (type    (func (result i32)))              ;; 3: void -> i32
  (type    (func (param) (result i32)))      ;; 4: void -> i32
  (type $T (func (param i32) (result i32)))  ;; 5: i32 -> i32
  (type $U (func (param i32)))               ;; 6: i32 -> void

  (func $print (import "spectest" "print_i32") (type 6))

  (func (type 0))
  (func (type $S))

  (func (export "one") (type 4) (i32.const 13))
  (func (export "two") (type $T) (i32.add (get_local 0) (i32.const 1)))

  ;; Both signature and parameters are allowed (and required to match)
  ;; since this allows the naming of parameters.
  (func (export "three") (type $T) (param $a i32) (result i32)
    (i32.sub (get_local 0) (i32.const 2))
  )

  (func (export "four") (type $U) (call $print (get_local 0)))
)

(invoke "four" (i32.const 83))

