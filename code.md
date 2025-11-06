```rust
#![allow(dead_code, mutable_transmutes, non_camel_case_types, non_snake_case, non_upper_case_globals, unused_assignments, unused_mut)]
#[no_mangle]
pub unsafe extern "C" fn atoi(mut str: *mut libc::c_char) -> libc::c_int {
    let mut result: libc::c_int = 0 as libc::c_int;
    let mut sign: libc::c_int = 1 as libc::c_int;
    while *str as libc::c_int == ' ' as i32 || *str as libc::c_int == '\t' as i32
        || *str as libc::c_int == '\n' as i32 || *str as libc::c_int == '\r' as i32
        || *str as libc::c_int == '\u{b}' as i32 || *str as libc::c_int == '\u{c}' as i32
    {
        str = str.offset(1);
        str;
    }
    if *str as libc::c_int == '+' as i32 || *str as libc::c_int == '-' as i32 {
        if *str as libc::c_int == '-' as i32 {
            sign = -(1 as libc::c_int);
        }
        str = str.offset(1);
        str;
    }
    while *str as libc::c_int >= '0' as i32 && *str as libc::c_int <= '9' as i32 {
        result = result * 10 as libc::c_int + (*str as libc::c_int - '0' as i32);
        str = str.offset(1);
        str;
    }
    return sign * result;
}
```
