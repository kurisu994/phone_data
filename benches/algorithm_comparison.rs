use criterion::{black_box, criterion_group, criterion_main, Criterion};
use phone_data::{PhoneData, PhoneLookup};
use phone_data::phone_hash::PhoneDataHash;
use std::time::Duration;

// 加载二分法版本数据
fn load_binary_search_data() -> PhoneData {
    PhoneData::new().expect("Failed to load binary search phone data")
}

// 加载哈希法版本数据
fn load_hash_data() -> PhoneDataHash {
    PhoneDataHash::new().expect("Failed to load hash phone data")
}

fn bench_single_lookup_comparison(c: &mut Criterion) {
    let binary_data = load_binary_search_data();
    let hash_data = load_hash_data();

    let test_phones = vec![
        "18086834111",
        "13800138000",
        "15900000000",
        "18612345678",
        "13344445555",
        "17766668888",
        "14999990000",
        "19988887777",
        "13456789012",
        "13698765432",
    ];

    let mut group = c.benchmark_group("single_lookup_comparison");
    group.measurement_time(Duration::from_secs(10));

    // 二分法查找基准测试
    for phone in &test_phones {
        group.bench_with_input(
            format!("binary_search_{}", phone),
            black_box(phone),
            |b, phone| {
                b.iter(|| {
                    black_box(binary_data.find(black_box(phone)).ok());
                })
            },
        );
    }

    // 哈希法查找基准测试
    for phone in &test_phones {
        group.bench_with_input(
            format!("hash_lookup_{}", phone),
            black_box(phone),
            |b, phone| {
                b.iter(|| {
                    black_box(hash_data.find(black_box(phone)).ok());
                })
            },
        );
    }

    group.finish();
}

fn bench_bulk_lookup_comparison(c: &mut Criterion) {
    let binary_data = load_binary_search_data();
    let hash_data = load_hash_data();

    // 生成测试用的手机号码
    let test_phones: Vec<String> = (13000000000i64..=18999999999i64)
        .step_by(1234567)
        .map(|x| x.to_string())
        .take(1000) // 只取前1000个号码进行测试
        .collect();

    c.bench_function("binary_search_bulk_1000", |b| {
        b.iter(|| {
            for phone in &test_phones {
                black_box(binary_data.find(phone).ok());
            }
        })
    });

    c.bench_function("hash_lookup_bulk_1000", |b| {
        b.iter(|| {
            for phone in &test_phones {
                black_box(hash_data.find(phone).ok());
            }
        })
    });
}

fn bench_initialization_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("initialization_time");
    group.measurement_time(Duration::from_secs(5));

    // 测试二分法数据初始化时间
    group.bench_function("binary_search_init", |b| {
        b.iter(|| {
            black_box(PhoneData::new().ok());
        })
    });

    // 测试哈希法数据初始化时间
    group.bench_function("hash_init", |b| {
        b.iter(|| {
            black_box(PhoneDataHash::new().ok());
        })
    });

    group.finish();
}

fn bench_memory_efficiency(c: &mut Criterion) {
    let binary_data = load_binary_search_data();
    let hash_data = load_hash_data();

    let mut group = c.benchmark_group("memory_efficiency");
    group.measurement_time(Duration::from_secs(5));

    // 模拟内存访问模式测试
    let test_phone = "18086834111";

    group.bench_function("binary_search_memory_access", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(binary_data.find(black_box(test_phone)).ok());
            }
        })
    });

    group.bench_function("hash_memory_access", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(hash_data.find(black_box(test_phone)).ok());
            }
        })
    });

    group.finish();
}

fn bench_failed_lookups(c: &mut Criterion) {
    let binary_data = load_binary_search_data();
    let hash_data = load_hash_data();

    let mut group = c.benchmark_group("failed_lookups");
    group.measurement_time(Duration::from_secs(5));

    // 使用不存在的手机号测试失败查找的性能
    let invalid_phones = vec![
        "99999999999",  // 不存在的号段
        "12345678901",  // 不存在的号段
        "11111111111",  // 不存在的号段
        "00000000000",  // 不存在的号段
    ];

    for phone in &invalid_phones {
        group.bench_with_input(
            format!("binary_search_failed_{}", phone),
            black_box(phone),
            |b, phone| {
                b.iter(|| {
                    black_box(binary_data.find(black_box(phone)).ok());
                })
            },
        );

        group.bench_with_input(
            format!("hash_failed_{}", phone),
            black_box(phone),
            |b, phone| {
                b.iter(|| {
                    black_box(hash_data.find(black_box(phone)).ok());
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_single_lookup_comparison,
    bench_bulk_lookup_comparison,
    bench_initialization_time,
    bench_memory_efficiency,
    bench_failed_lookups
);
criterion_main!(benches);