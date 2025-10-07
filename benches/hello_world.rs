fn main() {
    divan::main();
}

fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

#[divan::bench]
fn my_first_bench() {
    fibonacci(15);
}
