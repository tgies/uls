//! Criterion benchmarks for database hot paths.
//!
//! Benchmarks the critical database operations used during ULS data import and querying.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusqlite::Connection;
use uls_core::records::{AmateurRecord, EntityRecord, HeaderRecord, UlsRecord};
use uls_db::{BulkInserter, Schema};

// Test fixtures matching FCC record structure
const TEST_USI: &str = "100001";
const TEST_CALLSIGN: &str = "W1BENCH";

fn create_header() -> HeaderRecord {
    HeaderRecord::from_fields(&[
        "HD",
        TEST_USI,
        "0000000001",
        "",
        TEST_CALLSIGN,
        "A",
        "HA",
        "01/15/2020",
        "01/15/2030",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "N",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "01/15/2020",
        "01/15/2020",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
    ])
}

fn create_entity() -> EntityRecord {
    EntityRecord::from_fields(&[
        "EN",
        TEST_USI,
        "",
        "",
        TEST_CALLSIGN,
        "L",
        "L00100001",
        "DOE, JOHN A",
        "JOHN",
        "A",
        "DOE",
        "",
        "555-555-1234",
        "",
        "bench@example.com",
        "123 Main St",
        "ANYTOWN",
        "CA",
        "90210",
        "",
        "",
        "000",
        "0001234567",
        "I",
        "",
        "",
        "",
        "",
        "",
        "",
    ])
}

fn create_amateur() -> AmateurRecord {
    AmateurRecord::from_fields(&[
        "AM",
        TEST_USI,
        "",
        "",
        TEST_CALLSIGN,
        "E",
        "D",
        "6",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
    ])
}

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    Schema::initialize(&conn).unwrap();
    conn
}

/// Benchmark BulkInserter::new() - prepared statement creation
fn bench_bulk_inserter_new(c: &mut Criterion) {
    let conn = setup_db();

    c.bench_function("bulk_inserter_new", |b| {
        b.iter(|| black_box(BulkInserter::new(black_box(&conn)).unwrap()))
    });
}

/// Benchmark inserting a single header record
fn bench_insert_header(c: &mut Criterion) {
    let conn = setup_db();
    let record = UlsRecord::Header(create_header());

    c.bench_function("insert_header", |b| {
        let mut inserter = BulkInserter::new(&conn).unwrap();
        b.iter(|| inserter.insert(black_box(&record)).unwrap())
    });
}

/// Benchmark inserting a full license set (header + entity + amateur)
fn bench_insert_license_set(c: &mut Criterion) {
    let conn = setup_db();
    let header = UlsRecord::Header(create_header());
    let entity = UlsRecord::Entity(create_entity());
    let amateur = UlsRecord::Amateur(create_amateur());

    c.bench_function("insert_license_set", |b| {
        let mut inserter = BulkInserter::new(&conn).unwrap();
        b.iter(|| {
            inserter.insert(black_box(&header)).unwrap();
            inserter.insert(black_box(&entity)).unwrap();
            inserter.insert(black_box(&amateur)).unwrap();
        })
    });
}

/// Benchmark callsign lookup query
fn bench_lookup_callsign(c: &mut Criterion) {
    let conn = setup_db();

    // Insert test data
    let mut inserter = BulkInserter::new(&conn).unwrap();
    inserter
        .insert(&UlsRecord::Header(create_header()))
        .unwrap();
    inserter
        .insert(&UlsRecord::Entity(create_entity()))
        .unwrap();
    inserter
        .insert(&UlsRecord::Amateur(create_amateur()))
        .unwrap();
    drop(inserter);

    c.bench_function("lookup_callsign", |b| {
        b.iter(|| {
            // Re-prepare each iteration to benchmark the full lookup path
            let mut stmt = conn
                .prepare_cached("SELECT call_sign FROM licenses WHERE call_sign = ? COLLATE NOCASE")
                .unwrap();
            let result: Option<String> = stmt
                .query_row([black_box(TEST_CALLSIGN)], |row| row.get(0))
                .ok();
            black_box(result)
        })
    });
}

criterion_group!(
    benches,
    bench_bulk_inserter_new,
    bench_insert_header,
    bench_insert_license_set,
    bench_lookup_callsign,
);
criterion_main!(benches);
