use benchmarks::basic::basic;
use criterion::criterion_main;

mod benchmarks;

criterion_main!(basic);
