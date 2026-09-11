"""Exercise the benchmark's independent database comparison oracle."""

import importlib.util
from pathlib import Path
import sqlite3
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("compare_imports", Path(__file__).with_name("compare-imports.py"))
benchmark = importlib.util.module_from_spec(spec)
spec.loader.exec_module(benchmark)


class DatabaseComparisonTest(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.database = Path(self.directory.name) / "import.db"
        self.execute("""
            CREATE TABLE licenses(id INTEGER PRIMARY KEY, name TEXT, expired_date TEXT);
            CREATE INDEX license_name ON licenses(name);
            CREATE TABLE import_status(service TEXT PRIMARY KEY, imported_at TEXT);
            CREATE TABLE metadata(key TEXT PRIMARY KEY, value TEXT);
            INSERT INTO licenses VALUES(1, 'O''Brien | Émile', '2030-01-01');
            INSERT INTO import_status VALUES('HA', '2026-09-11T12:00:00+00:00');
            INSERT INTO metadata VALUES('schema_version', '8');
        """)

    def execute(self, sql):
        connection = sqlite3.connect(self.database)
        try:
            connection.executescript(sql)
        finally:
            connection.close()

    def test_detects_a_single_changed_fcc_value(self):
        before = benchmark.verify_database(self.database)
        self.execute("UPDATE licenses SET expired_date='2031-01-01' WHERE id=1;")
        after = benchmark.verify_database(self.database)
        self.assertNotEqual(before, after)
        self.assertNotEqual(before["tables"]["licenses"]["sha3_256"], after["tables"]["licenses"]["sha3_256"])

    def test_checks_schema_and_preserves_only_runtime_timestamp_exception(self):
        before = benchmark.verify_database(self.database)
        self.execute("UPDATE import_status SET imported_at='2026-09-12T15:00:00+00:00';")
        self.assertEqual(before, benchmark.verify_database(self.database))
        self.execute("DROP INDEX license_name;")
        self.assertNotEqual(before, benchmark.verify_database(self.database))

    def test_refuses_nonempty_wal(self):
        Path(str(self.database) + "-wal").write_bytes(b"uncheckpointed data")
        with self.assertRaises(AssertionError):
            benchmark.verify_database(self.database)


if __name__ == "__main__":
    unittest.main()
