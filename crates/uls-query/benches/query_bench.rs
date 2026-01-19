//! Criterion benchmarks for query engine hot paths.
//!
//! Benchmarks the critical query operations used during lookups and searches.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uls_query::{SearchFilter, SortOrder};

/// Benchmark SearchFilter construction
fn bench_filter_construction(c: &mut Criterion) {
    c.bench_function("filter_name", |b| {
        b.iter(|| black_box(SearchFilter::name(black_box("SMITH"))))
    });

    c.bench_function("filter_callsign", |b| {
        b.iter(|| black_box(SearchFilter::callsign(black_box("W1AW"))))
    });

    c.bench_function("filter_location", |b| {
        b.iter(|| {
            black_box(SearchFilter::location(
                Some("LOS ANGELES".to_string()),
                Some("CA".to_string()),
            ))
        })
    });
}

/// Benchmark filter chaining (common usage pattern)
fn bench_filter_chaining(c: &mut Criterion) {
    c.bench_function("filter_chain_state", |b| {
        b.iter(|| {
            let filter = SearchFilter::new().with_state(black_box("CA"));
            black_box(filter)
        })
    });

    c.bench_function("filter_chain_with_sort", |b| {
        b.iter(|| {
            let filter = SearchFilter::new()
                .with_state(black_box("CA"))
                .with_sort(SortOrder::CallSign);
            black_box(filter)
        })
    });

    c.bench_function("filter_chain_full", |b| {
        b.iter(|| {
            let filter = SearchFilter::name(black_box("SMITH"))
                .with_state(black_box("CA"))
                .with_limit(100)
                .with_sort(SortOrder::Name);
            black_box(filter)
        })
    });
}

/// Benchmark SQL clause generation (important for query performance)
fn bench_sql_generation(c: &mut Criterion) {
    let simple_filter = SearchFilter::name("SMITH");
    let complex_filter = SearchFilter::name("SMITH")
        .with_state("CA")
        .with_limit(100)
        .with_sort(SortOrder::CallSign);

    c.bench_function("where_clause_simple", |b| {
        b.iter(|| black_box(simple_filter.to_where_clause()))
    });

    c.bench_function("where_clause_complex", |b| {
        b.iter(|| black_box(complex_filter.to_where_clause()))
    });

    c.bench_function("order_clause", |b| {
        b.iter(|| black_box(complex_filter.order_clause()))
    });
}

criterion_group!(
    benches,
    bench_filter_construction,
    bench_filter_chaining,
    bench_sql_generation,
);
criterion_main!(benches);
