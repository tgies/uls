//! Criterion benchmarks for parser hot paths.
//!
//! Benchmarks the critical parsing operations used during ULS data import.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use uls_parser::{DatReader, ParsedLine};

// Realistic sample lines from actual ULS data
const SAMPLE_HEADER: &str = "HD|3941279|0004312345||W1AW|A|HA|01/15/2020|01/15/2030|||||||||||||||||||||||||||||||||||N|||||||||01/15/2020|01/15/2020|||||||||||";
const SAMPLE_ENTITY: &str = "EN|3941279|0004312345||W1AW|L|L00123456|ARRL INC|ARRL|I|THE|INC|860-594-0200||hq@arrl.org|225 MAIN STREET|NEWINGTON|CT|06111||||0001234567|B|||||||";
const SAMPLE_AMATEUR: &str = "AM|3941279|0004312345||W1AW|E|D|1||||||||||||";
const SAMPLE_HISTORY: &str = "HS|3941279|0004312345|W1AW|01/15/2020|LIISS";
const SAMPLE_COMMENT: &str = "CO|3941279|0004312345|W1AW|01/15/2020|ARRL Headquarters Station - special exemption for operation|A|01/15/2020";

/// Benchmark ParsedLine::from_line for different record types
fn bench_parse_line(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_line");

    let samples = [
        ("HD_header", SAMPLE_HEADER),
        ("EN_entity", SAMPLE_ENTITY),
        ("AM_amateur", SAMPLE_AMATEUR),
        ("HS_history", SAMPLE_HISTORY),
        ("CO_comment", SAMPLE_COMMENT),
    ];

    for (name, line) in samples {
        group.throughput(Throughput::Bytes(line.len() as u64));
        group.bench_with_input(BenchmarkId::new("from_line", name), &line, |b, &line| {
            b.iter(|| black_box(ParsedLine::from_line(black_box(line), 1).unwrap()))
        });
    }

    group.finish();
}

/// Benchmark to_record conversion (parsing + typed construction)
fn bench_to_record(c: &mut Criterion) {
    let mut group = c.benchmark_group("to_record");

    let samples = [
        ("HD_header", SAMPLE_HEADER),
        ("EN_entity", SAMPLE_ENTITY),
        ("AM_amateur", SAMPLE_AMATEUR),
        ("HS_history", SAMPLE_HISTORY),
        ("CO_comment", SAMPLE_COMMENT),
    ];

    for (name, line) in samples {
        // Pre-parse the line, then benchmark just the to_record conversion
        let parsed = ParsedLine::from_line(line, 1).unwrap();

        group.bench_with_input(BenchmarkId::new("convert", name), &parsed, |b, parsed| {
            b.iter(|| black_box(parsed.to_record().unwrap()))
        });
    }

    group.finish();
}

/// Benchmark full pipeline: parsing + record conversion
fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    let samples = [
        ("HD_header", SAMPLE_HEADER),
        ("EN_entity", SAMPLE_ENTITY),
        ("AM_amateur", SAMPLE_AMATEUR),
    ];

    for (name, line) in samples {
        group.throughput(Throughput::Bytes(line.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("parse_and_convert", name),
            &line,
            |b, &line| {
                b.iter(|| {
                    let parsed = ParsedLine::from_line(black_box(line), 1).unwrap();
                    black_box(parsed.to_record().unwrap())
                })
            },
        );
    }

    group.finish();
}

/// Benchmark DatReader iteration over in-memory data
fn bench_dat_reader(c: &mut Criterion) {
    // Simulate a small DAT file with multiple records
    let data = format!(
        "{}\n{}\n{}\n{}\n{}\n",
        SAMPLE_HEADER, SAMPLE_ENTITY, SAMPLE_AMATEUR, SAMPLE_HISTORY, SAMPLE_COMMENT
    );

    c.bench_function("dat_reader_5_records", |b| {
        b.iter(|| {
            let reader = DatReader::new(black_box(data.as_bytes()));
            let records: Vec<_> = reader.collect();
            black_box(records)
        })
    });
}

criterion_group!(
    benches,
    bench_parse_line,
    bench_to_record,
    bench_full_pipeline,
    bench_dat_reader,
);
criterion_main!(benches);
