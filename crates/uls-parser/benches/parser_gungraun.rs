//! Gungraun (Valgrind-based) benchmarks for parser operations.
//!
//! These benchmarks count CPU instructions for deterministic, reproducible results.

use gungraun::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;
use uls_parser::ParsedLine;

// Sample lines for benchmarking
const SAMPLE_HEADER: &str = "HD|3941279|0004312345||W1AW|A|HA|01/15/2020|01/15/2030|||||||||||||||||||||||||||||||||||N|||||||||01/15/2020|01/15/2020|||||||||||";
const SAMPLE_ENTITY: &str = "EN|3941279|0004312345||W1AW|L|L00123456|ARRL INC|ARRL|I|THE|INC|860-594-0200||hq@arrl.org|225 MAIN STREET|NEWINGTON|CT|06111||||0001234567|B|||||||";
const SAMPLE_AMATEUR: &str = "AM|3941279|0004312345||W1AW|E|D|1||||||||||||";

#[library_benchmark]
fn parse_header_line() -> ParsedLine {
    black_box(ParsedLine::from_line(black_box(SAMPLE_HEADER), 1).unwrap())
}

#[library_benchmark]
fn parse_entity_line() -> ParsedLine {
    black_box(ParsedLine::from_line(black_box(SAMPLE_ENTITY), 1).unwrap())
}

#[library_benchmark]
fn parse_amateur_line() -> ParsedLine {
    black_box(ParsedLine::from_line(black_box(SAMPLE_AMATEUR), 1).unwrap())
}

#[library_benchmark]
fn parse_and_convert_header() {
    let parsed = ParsedLine::from_line(black_box(SAMPLE_HEADER), 1).unwrap();
    black_box(parsed.to_record().unwrap());
}

#[library_benchmark]
fn parse_and_convert_entity() {
    let parsed = ParsedLine::from_line(black_box(SAMPLE_ENTITY), 1).unwrap();
    black_box(parsed.to_record().unwrap());
}

library_benchmark_group!(
    name = parser_benchmarks;
    benchmarks =
        parse_header_line,
        parse_entity_line,
        parse_amateur_line,
        parse_and_convert_header,
        parse_and_convert_entity,
);

main!(library_benchmark_groups = parser_benchmarks);
