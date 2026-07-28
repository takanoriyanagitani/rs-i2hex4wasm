pub fn int2hex2string(i: i32) -> String {
    format!("{i:08x}")
}

pub fn int2hex2string2u64le(i: i32) -> u64 {
    let s: String = int2hex2string(i);
    let b: &[u8] = s.as_bytes();
    let a: [u8; 8] = b.try_into().unwrap_or_default();
    u64::from_le_bytes(a)
}

#[cfg(test)]
mod tests {

    mod int2hex2string {

        #[test]
        fn zero() {
            let i: i32 = 0;
            let s: String = crate::int2hex2string(i);
            assert_eq!(s, "00000000");
        }

        #[test]
        fn one() {
            let i: i32 = 1;
            let s: String = crate::int2hex2string(i);
            assert_eq!(s, "00000001");
        }
    }

    mod int2hex2string2u64le {

        #[test]
        fn zero() {
            let i: i32 = 0;
            let u: u64 = crate::int2hex2string2u64le(i);
            assert_eq!(u, 0x3030_3030_3030_3030);
        }

        #[test]
        fn one() {
            let i: i32 = 1;
            let u: u64 = crate::int2hex2string2u64le(i);
            assert_eq!(u, 0x3130_3030_3030_3030);
        }
    }
}
