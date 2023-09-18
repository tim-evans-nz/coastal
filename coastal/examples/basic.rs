use coastal::coast;

#[coast]
pub const VERSION_MAJOR: u32 = 1;
#[coast]
pub const VERSION_MINOR: u32 = 0;
#[coast]
pub const VERSION: &str = "1.0";

#[coast]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

coastal::api! {
    const VERSION_MAJOR;
    const VERSION_MINOR;
    const VERSION;
    fn add;
}
