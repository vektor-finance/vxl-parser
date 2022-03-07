use criterion::{black_box, criterion_group, Criterion};

use core::parse;

const TEST_SIMPLE: &str = r#"
FUNCTION.SUBFUNCTION(ARG1, 10, FALSE)
"#;

pub fn simple(c: &mut Criterion) {
  c.bench_function("Parse VXL (simple)", |b| b.iter(|| parse(black_box(TEST_SIMPLE))));
}

criterion_group!(basic, simple);
