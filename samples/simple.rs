unsafe extern "C" {
    safe fn print(value: i32);
}

fn main() {
    print(42);
}
