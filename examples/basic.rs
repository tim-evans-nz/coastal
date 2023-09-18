use coastal::{coastal, coastal_impl};

#[coastal]
pub const VERSION_MAJOR: u32 = 1;
#[coastal]
pub const VERSION_MINOR: u32 = 0;
#[coastal]
pub const VERSION: &str = "1.0";

#[coastal]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

coastal_impl! {
    const VERSION_MAJOR;
    const VERSION_MINOR;
    const VERSION;
    fn add;
}
