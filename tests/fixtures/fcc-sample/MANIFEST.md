# FCC ULS Test Fixture

This directory contains a representative subset of FCC ULS data for testing.
All records are real FCC data with referential integrity preserved.

## Generation

```bash
python scripts/extract_test_fixture.py <cache_dir> <output_dir> --count 20
```

## Contents

### l_amat

| File | Records |
|------|--------|
| AM.dat | 34 |
| CO.dat | 7 |
| EN.dat | 34 |
| HD.dat | 34 |
| HS.dat | 114 |
| LA.dat | 1 |
| SC.dat | 7 |

### l_gmrs

| File | Records |
|------|--------|
| CO.dat | 5 |
| EN.dat | 33 |
| HD.dat | 33 |
| HS.dat | 71 |
| SC.dat | 15 |

**Total records:** 388
