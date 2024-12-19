use common::ecalls;

fn main() {
    ecalls::foo::ecall();
    println!("Hello, world!");
}
