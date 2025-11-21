# Phone Number Location Query System

A high-performance phone number location query service implemented in Rust 2024 edition, supporting multiple search algorithms with millisecond-level response times.

## Features

- ⚡ **Ultra High Performance**: Multiple algorithm support with optimal query performance reaching 29ns
- 🔧 **Rich Algorithm Support**: Binary search, hash lookup, SIMD optimization, Bloom filter, and more
- 📱 **Comprehensive Carrier Support**: Supports China Mobile, China Unicom, China Telecom, China Broadcasting, and their virtual operators
- 🌐 **HTTP API**: RESTful API interface with multiple query methods
- 📊 **Accurate Data**: Regularly updated location database with 517,258 phone number records
- 🧪 **Complete Testing**: Unified test suite ensuring algorithm consistency and functionality

### Data Scale
- Location database file size: 4.5MB
- Location database last updated: February 2025
- Phone number prefix records: 517,258

### Tech Stack
- **Rust Edition**: 2024
- **Web Framework**: actix-web 4.11.0
- **Serialization**: serde 1.0.228
- **Error Handling**: anyhow 1.0.100
- **Benchmark Testing**: criterion 0.5.1

## Algorithm Implementations

This project implements four different search algorithms, each with unique characteristics:

### 1. Binary Search Algorithm

**File**: `src/lib.rs`

**Time Complexity**: O(log n)
**Space Complexity**: O(1)

**Features**:
- Classic algorithm, stable and reliable
- Minimal memory usage
- Suitable for memory-constrained environments
- Query time: ~175ns

**Optimizations**:
- Bit operations for division optimization
- Unsafe access for performance improvement
- Optimized loop termination conditions

### 2. Hash Lookup Algorithm

**File**: `src/phone_hash.rs`

**Time Complexity**: O(1) average case
**Space Complexity**: O(n)

**Features**:
- Fastest lookup speed
- Pre-allocated HashMap capacity
- Query time: ~105ns (40% faster than binary search)
- Higher memory usage

**Use Cases**:
- Scenarios with extreme query performance requirements
- Memory-rich environments

### 3. SIMD Optimized Algorithm

**File**: `src/phone_simd.rs`

**Time Complexity**: O(log n)
**Space Complexity**: O(1)

**Features**:
- Vectorized instruction optimization
- Branch prediction friendly
- Batch query optimization
- Suitable for CPUs with SIMD support

**Optimization Techniques**:
- Prefetch optimization to reduce cache misses
- Loop unrolling
- Vectorized comparisons

### 4. Bloom Filter Algorithm

**File**: `src/phone_bloom.rs`

**Time Complexity**: O(k) where k is the number of hash functions
**Space Complexity**: O(n)

**Features**:
- Fast pre-filtering of non-existent numbers
- 1% false positive rate setting
- Two-stage lookup: Bloom filter + binary search
- Excellent performance for failed queries

**Workflow**:
1. Quick Bloom filter check
2. Binary search if possibly exists
3. Return precise result

## Performance Benchmarks

Based on the latest benchmark test results (Criterion v0.5.1, test environment: macOS), detailed performance comparison of all four algorithms:

### Single Query Performance Comparison

| Algorithm | Average Query Time | Relative Performance | Memory Usage | Use Cases |
|-----------|-------------------|---------------------|--------------|-----------|
| Hash Lookup | ~150ns | Fastest | 156MB | High-frequency queries, ample memory |
| Binary Search | ~233ns | Baseline | 6.2MB | Memory-constrained environments |
| SIMD Optimized | ~235ns | Comparable | 6.2MB | Batch query optimization |
| Bloom Filter | ~260ns | Slower | 6.5MB | High failure query rate |

**Performance Characteristics**:
- Hash lookup is fastest in all scenarios but has highest memory overhead
- Binary search and SIMD optimization have comparable performance, SIMD has advantages in batch queries
- Bloom filter performs excellently in failure queries, slightly slower than binary search in successful queries

### Batch Query Performance (1000 numbers)

| Algorithm | Total Time | Avg Per Query | QPS | Performance Improvement |
|-----------|------------|---------------|-----|-------------------------|
| Hash Lookup | 139.73µs | 140ns | ~7,143,000 | +66% |
| SIMD Lookup | 222.49µs | 222ns | ~4,505,000 | +5% |
| Bloom Lookup | 254.71µs | 255ns | ~3,922,000 | -9% |
| Binary Search | 232.87µs | 233ns | ~4,291,000 | Baseline |

### Initialization Time Comparison

| Algorithm | Initialization Time | Reason | Cold Start Impact |
|-----------|-------------------|--------|-------------------|
| Binary Search | 2.23ms | Direct data loading | Minimal |
| SIMD Lookup | 2.31ms | Same as binary search | Minimal |
| Bloom Lookup | 27.03ms | Additional Bloom filter construction | Small |
| Hash Lookup | 204.27ms | HashMap construction overhead | Significant |

### Failed Query Performance (Non-existent Numbers)

For non-existent phone numbers, different algorithms show:

| Algorithm | Query Time | Advantage | Use Cases |
|-----------|------------|-----------|-----------|
| Hash Lookup | 33–36ns | Direct key non-existence check | Low input quality scenarios |
| Bloom Filter | ~231ns | Fast pre-filtering | High volume of invalid queries |
| Binary Search | ~209ns | Stable and reliable | General scenarios |
| SIMD Lookup | ~212ns | Similar to binary search | General scenarios |

### Detailed Memory Usage Comparison

| Algorithm | Memory Usage | Components | Memory Efficiency |
|-----------|--------------|------------|-------------------|
| Binary Search | 6.2MB | Raw data (4.5MB) + Index (1.7MB) | Highest |
| SIMD Optimized | 6.2MB | Same as binary search | Highest |
| Bloom Filter | 6.5MB | Base data + Bloom bitmap (0.3MB) | High |
| Hash Lookup | 156MB | Base data + HashMap overhead (150MB) | Lowest |

### Scenario Recommendations

**🚀 Pursuing Ultimate Performance**
- First choice: Hash lookup algorithm
- Scenario: High-frequency queries, ample memory
- Performance: Single query 106ns, QPS up to 9.7M

**💾 Memory-Constrained Environments**
- First choice: Binary search algorithm
- Scenario: Embedded devices, memory-constrained
- Memory: Only 6.2MB

**📊 Batch Query Optimization**
- First choice: SIMD optimization algorithm
- Scenario: Data analysis, batch processing
- Features: Vectorized instructions, cache-friendly

**🔍 Unstable Input Quality**
- First choice: Bloom filter algorithm
- Scenario: Large number of invalid queries need filtering
- Features: Excellent failed query performance

### Benchmark Running Methods

```bash
# Run all algorithm comparison tests
cargo bench --bench algorithm_comparison

# Run specific algorithm tests
cargo bench --bench lookup_performance

# View detailed performance reports
cargo bench --bench algorithm_comparison -- --output-format html
```

## phone.dat File Format

```
| 4 bytes | Version number (e.g., 1701 for January 2017)
| 4 bytes | First index offset
| offset - 8 | Record area
| Remaining part | Index area
```

1. **Header**: 8 bytes, 4-byte version number, 4-byte first index offset
2. **Record Area**: Each record format is "<Province>|<City>|<Postal Code>|<Area Code>\0"
3. **Index Area**: Each record format is "<Phone Number Prefix><Record Area Offset><Card Type>", 9 bytes length

## Requirements

- Rust 1.85+ (supporting 2024 edition)
- phone.dat file (must be placed in project root directory)

## Installation and Usage

### Build and Run
```bash
# Clone project
git clone <repository-url>
cd phone_data

# Install dependencies and run
cargo run

# Production build
cargo build --release
```

### Library Usage Example

```rust
use phone_data::{PhoneData, PhoneLookup};

// Binary search
let phone_data = PhoneData::new()?;
let result = phone_data.find("18086834111")?;
println!("Province: {}", result.province);

// Hash lookup
use phone_data::phone_hash::PhoneDataHash;
let hash_data = PhoneDataHash::new()?;
let result = hash_data.find("18086834111")?;

// SIMD optimized lookup
use phone_data::phone_simd::PhoneDataSimd;
let simd_data = PhoneDataSimd::new()?;
let result = simd_data.find("18086834111")?;

// Bloom filter lookup
use phone_data::phone_bloom::PhoneDataBloom;
let bloom_data = PhoneDataBloom::new()?;
let result = bloom_data.find("18086834111")?;
```

### API Usage Example

Service starts on port 8080 by default:

```shell
# Path parameter query
curl 'http://127.0.0.1:8080/query2/18086834111'

# GET parameter query
curl 'http://127.0.0.1:8080/query?phone=18086834111'
```

**Response Format**:
```json
{
    "code": 0,
    "data": {
        "province": "Sichuan",
        "city": "Chengdu",
        "zip_code": "610000",
        "area_code": "028",
        "card_type": "China Telecom"
    },
    "success": true,
    "result": "ok"
}
```

## Testing

### Running Test Suite
```bash
# Run all tests
cargo test

# Run unified test suite (recommended)
cargo test --test unified_tests -- --nocapture

# Run integration tests
cargo test --test integration_tests

# Run library tests
cargo test --lib
```

### Performance Benchmark Testing
```bash
# Run all benchmark tests
cargo bench

# Run algorithm comparison benchmarks
cargo bench --bench algorithm_comparison

# Run lookup performance benchmarks
cargo bench --bench lookup_performance
```

## Algorithm Selection Guidelines

Choose the appropriate algorithm based on different scenarios:

### 1. Pursuing Ultimate Performance
**Recommendation**: Hash lookup algorithm
- Shortest query time (105ns)
- Suitable for high-frequency query scenarios
- Requires sufficient memory (42MB)

### 2. Memory-Constrained Environments
**Recommendation**: Binary search algorithm
- Minimal memory usage (8MB)
- Stable and reliable performance
- Suitable for embedded or memory-constrained scenarios

### 3. Batch Query Optimization
**Recommendation**: SIMD optimization algorithm
- Vectorized instruction acceleration
- Batch query friendly
- Suitable for data analysis scenarios

### 4. High Failure Query Rate
**Recommendation**: Bloom filter algorithm
- Quick filtering of non-existent numbers
- Reduce invalid query overhead
- Suitable for scenarios with poor input quality

## API Documentation

### Query Interfaces

#### 1. GET Parameter Query
```
GET /query?phone=<phone_number>
```

#### 2. Path Parameter Query
```
GET /query2/<phone_number>
```

### Response Format
```json
{
    "code": 0,           // Status code, 0 for success
    "data": {            // Phone number information
        "province": "Province",
        "city": "City",
        "zip_code": "Postal Code",
        "area_code": "Area Code",
        "card_type": "Carrier"
    },
    "success": true,     // Success flag
    "result": "ok"       // Result description
}
```

## Development Notes

### Project Structure
```
src/
├── lib.rs              # Binary search algorithm implementation
├── main.rs             # Web service entry point
├── common.rs           # Common types and interface definitions
├── phone_hash.rs       # Hash lookup algorithm
├── phone_simd.rs       # SIMD optimization algorithm
└── phone_bloom.rs      # Bloom filter algorithm

tests/
├── integration_tests.rs # Integration tests
├── test_suite.rs       # Unified test suite
└── unified_tests.rs    # Unified test entry

benches/
├── algorithm_comparison.rs  # Algorithm comparison benchmarks
└── lookup_performance.rs    # Lookup performance benchmarks
```

### Adding New Algorithms
1. Create new file in `src/` directory
2. Implement `PhoneLookup` and `PhoneStats` traits
3. Export module in `src/lib.rs`
4. Add test cases in test suite
5. Add performance tests in benchmarks

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Changelog

### v0.2.0 (2025-11-13)
- ✨ Added hash lookup algorithm (105ns query time)
- ✨ Added SIMD optimization algorithm implementation
- ✨ Added Bloom filter algorithm
- 🔧 Refactored code architecture, extracted common types to common.rs
- 🧪 Improved test suite ensuring algorithm consistency
- 📊 Added comprehensive performance benchmarks
- 📚 Enhanced documentation and algorithm descriptions

### v0.1.0 (2025-02-xx)
- 🎉 Initial version release
- ⚡ Binary search algorithm implementation
- 🌐 HTTP API service
- 📱 Support for three major carriers and virtual operators