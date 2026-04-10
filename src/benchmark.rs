use std::time::Instant;

#[allow(dead_code)]
pub fn benchmark<F>(name: &str, iterations: usize, mut f: F)
where
    F: FnMut(),
{
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let elapsed = start.elapsed();
    println!(
        "{}: {} iterations in {:.3}s ({:.3}ms per iteration)",
        name,
        iterations,
        elapsed.as_secs_f64(),
        elapsed.as_millis() as f64 / iterations as f64
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Aegis;

    #[test]
    fn bench_simple_execution() {
        let aegis = Aegis::new();
        benchmark("simple_script", 1000, || {
            aegis.execute("let x = 42; result(x)").ok();
        });
    }

    #[test]
    fn bench_loop_execution() {
        let aegis = Aegis::new();
        benchmark("loop_100", 100, || {
            aegis
                .execute("let sum = 0; for i in 0..100 { sum = sum + i; } result(sum)")
                .ok();
        });
    }
}
