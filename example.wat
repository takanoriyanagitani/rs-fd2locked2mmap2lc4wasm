(module

  (memory (export "memory") 1)

  (func $reset_mem (export "reset_mem")
      i32.const 0
      i32.const 0
      i32.const 65536
      memory.fill
  )

  (func $count_line_nosimd (export "count_line_nosimd") (result i64)
        (local $ptr i32)
        (local $cnt i64)

        i32.const 0 local.set $ptr
        i64.const 0 local.set $cnt

        loop
          local.get $ptr i32.load8_u i32.const 0x0a i32.eq if
            i64.const 1 local.get $cnt i64.add local.set $cnt
          end
          local.get $ptr i32.const 1 i32.add local.tee $ptr
          i32.const 65536 i32.lt_u br_if 0
        end

        local.get $cnt
  )

  (func $count_line_simd (export "count_line_simd") (result i64)
        (local $ptr i32)
        (local $cnt i64)

        i32.const 0 local.set $ptr
        i64.const 0 local.set $cnt

        loop
          local.get $ptr v128.load
          v128.const i64x2 0x0a0a_0a0a_0a0a_0a0a 0x0a0a_0a0a_0a0a_0a0a
          i8x16.eq
          i8x16.bitmask
          i32.popcnt
          i64.extend_i32_u
          local.get $cnt
          i64.add
          local.set $cnt

          local.get $ptr i32.const 16 i32.add local.tee $ptr
          i32.const 65536 i32.lt_u br_if 0
        end

        local.get $cnt
  )

  (func $count_line_simd_unroll8 (export "count_line_simd_unroll8") (result i64)
        (local $ptr i32)
        (local $cnt i64)

        i32.const 0 local.set $ptr
        i64.const 0 local.set $cnt

        loop
          ;; vec 0
          local.get $ptr v128.load offset=0
          v128.const i64x2 0x0a0a_0a0a_0a0a_0a0a 0x0a0a_0a0a_0a0a_0a0a
          i8x16.eq
          i8x16.bitmask
          i32.popcnt

          ;; vec 1
          local.get $ptr v128.load offset=16
          v128.const i64x2 0x0a0a_0a0a_0a0a_0a0a 0x0a0a_0a0a_0a0a_0a0a
          i8x16.eq
          i8x16.bitmask
          i32.popcnt

          ;; vec 2
          local.get $ptr v128.load offset=32
          v128.const i64x2 0x0a0a_0a0a_0a0a_0a0a 0x0a0a_0a0a_0a0a_0a0a
          i8x16.eq
          i8x16.bitmask
          i32.popcnt

          ;; vec 3
          local.get $ptr v128.load offset=48
          v128.const i64x2 0x0a0a_0a0a_0a0a_0a0a 0x0a0a_0a0a_0a0a_0a0a
          i8x16.eq
          i8x16.bitmask
          i32.popcnt

          ;; vec 4
          local.get $ptr v128.load offset=64
          v128.const i64x2 0x0a0a_0a0a_0a0a_0a0a 0x0a0a_0a0a_0a0a_0a0a
          i8x16.eq
          i8x16.bitmask
          i32.popcnt

          ;; vec 5
          local.get $ptr v128.load offset=80
          v128.const i64x2 0x0a0a_0a0a_0a0a_0a0a 0x0a0a_0a0a_0a0a_0a0a
          i8x16.eq
          i8x16.bitmask
          i32.popcnt

          ;; vec 6
          local.get $ptr v128.load offset=96
          v128.const i64x2 0x0a0a_0a0a_0a0a_0a0a 0x0a0a_0a0a_0a0a_0a0a
          i8x16.eq
          i8x16.bitmask
          i32.popcnt

          ;; vec 7
          local.get $ptr v128.load offset=112
          v128.const i64x2 0x0a0a_0a0a_0a0a_0a0a 0x0a0a_0a0a_0a0a_0a0a
          i8x16.eq
          i8x16.bitmask
          i32.popcnt

          i32.add
          i32.add i32.add
          i32.add i32.add
          i32.add i32.add

          i64.extend_i32_u
          local.get $cnt
          i64.add
          local.set $cnt

          local.get $ptr i32.const 128 i32.add local.tee $ptr
          i32.const 65536 i32.lt_u br_if 0
        end

        local.get $cnt
  )

  (func $count_line (export "count_line") (result i64)
    call $count_line_simd_unroll8
  )

)
