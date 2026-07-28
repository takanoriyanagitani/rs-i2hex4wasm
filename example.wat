(module

  (import "host" "reset_omap" (func $reset_omap (result i32)))

  (import "host" "int2hex2u64" (func $int2hex2u64 (param i32) (result i64)))

  (import "host" "delete_key"
    (func $delete_key (param $ptr i32)(param $len i32) (result i32)))

  (import "host" "get_int_by_key"
    (func $get_int_by_key
      (param $ptr i32)
      (param $len i32)
      (result i32 i32) ;; e.g., ok -> (42, 0); ng -> (0, -1)
  ))

  (import "host" "overwrite_long_as_hex"
    (func $overwrite_long_as_hex
      (param $ptr i32)
      (param $len i32)
      (param $val i64)
      (result i32)
  ))

  (memory (export "memory") 2)

  (data (i32.const 0x0001_0000) "helo")

  (global $KEY_HELO i32 (i32.const 0x0001_0000))
  (global $LEN_HELO i32 (i32.const 4))

  (data (i32.const 0x0001_1000) "id")

  (global $KEY_ID i32 (i32.const 0x0001_1000))
  (global $LEN_ID i32 (i32.const 2))

  (func $converter (export "converter")
      (local $ret i32)
      (local $val i32)
      (local $hex i64)
      ;;global.get $KEY_HELO global.get $LEN_HELO call $delete_key drop
      ;;global.get $KEY_ID global.get $LEN_ID call $delete_key drop

      global.get $KEY_ID global.get $LEN_ID call $get_int_by_key
      local.set $ret local.set $val
      local.get $ret i32.const 0 i32.ne if
        unreachable
      end

      local.get $val call $int2hex2u64 local.set $hex

      global.get $KEY_ID global.get $LEN_ID local.get $hex
      call $overwrite_long_as_hex local.set $ret
      local.get $ret i32.const 0 i32.ne if
        unreachable
      end
  )

)
