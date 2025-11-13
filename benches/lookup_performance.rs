use criterion::{black_box, criterion_group, criterion_main, Criterion};
use phone_data::{PhoneData, PhoneLookup};
use std::time::Duration;

fn load_phone_data() -> PhoneData {
    PhoneData::new().expect("Failed to load phone data")
}

fn bench_find_performance(c: &mut Criterion) {
    let phone_data = load_phone_data();
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

    let mut group = c.benchmark_group("phone_lookup");
    group.measurement_time(Duration::from_secs(10));

    for phone in &test_phones {
        group.bench_with_input(
            format!("lookup_{}", phone),
            black_box(phone),
            |b, phone| {
                b.iter(|| {
                    black_box(phone_data.find(black_box(phone)).ok());
                })
            },
        );
    }

    group.finish();
}

fn bench_bulk_lookup(c: &mut Criterion) {
    let phone_data = load_phone_data();
    let test_phones: Vec<String> = (13000000000i64..=18999999999i64)
        .step_by(1234567)
        .map(|x| x.to_string())
        .collect();

    c.bench_function("bulk_lookup_1000_phones", |b| {
        b.iter(|| {
            for phone in &test_phones {
                black_box(phone_data.find(phone).ok());
            }
        })
    });
}

fn bench_phone_parsing(c: &mut Criterion) {
    let test_phones = vec![
        "18086834111",
        "13800138000",
        "15900000000",
        "18612345678",
        "13344445555",
    ];

    c.bench_function("phone_prefix_parsing", |b| {
        b.iter(|| {
            for phone in &test_phones {
                let prefix = if phone.len() == 7 {
                    black_box(phone.parse::<i32>().ok())
                } else {
                    black_box(phone[..7].parse::<i32>().ok())
                };
                black_box(prefix);
            }
        })
    });
}

criterion_group!(
    benches,
    bench_find_performance,
    bench_bulk_lookup,
    bench_phone_parsing
);
criterion_main!(benches);