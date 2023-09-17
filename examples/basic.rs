use coastal::{coastal, coastal_impl};

#[coastal]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

coastal_impl! {
    add
}
