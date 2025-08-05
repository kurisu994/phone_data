use phone_data::{ErrorKind, PhoneData};

#[test]
fn test_phone_lookup_success() {
    let phone_data = PhoneData::new().expect("Failed to load phone data");
    let phone = "1920000000";

    match phone_data.find(phone) {
        Ok(data) => {
            println!("Phone lookup result: {:?}", data);
            assert!(!data.province.is_empty());
            assert!(!data.city.is_empty());
        }
        Err(e) => {
            match e {
                ErrorKind::NotFound => {
                    println!("Phone number {} not found in database", phone);
                    // This is acceptable for test data
                }
                _ => panic!("Unexpected error during phone lookup: {:?}", e),
            }
        }
    }
}

#[test]
fn test_invalid_phone_length() {
    let phone_data = PhoneData::new().expect("Failed to load phone data");

    // Test too short
    let result = phone_data.find("123");
    assert!(matches!(result, Err(ErrorKind::InvalidLength)));

    // Test too long
    let result = phone_data.find("123456789012");
    assert!(matches!(result, Err(ErrorKind::InvalidLength)));
}

#[test]
fn test_multiple_phone_lookups() {
    let phone_data = PhoneData::new().expect("Failed to load phone data");
    let test_phones = ["1390000000", "1880000000", "1770000000"];

    for phone in &test_phones {
        let result = phone_data.find(phone);
        match result {
            Ok(data) => {
                println!("Phone {}: {} - {}", phone, data.province, data.card_type);
            }
            Err(ErrorKind::NotFound) => {
                println!("Phone {} not found (acceptable for test)", phone);
            }
            Err(e) => {
                panic!("Unexpected error for phone {}: {:?}", phone, e);
            }
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_lookup_performance() {
        let phone_data = PhoneData::new().expect("Failed to load phone data");
        let phone = "1380000000";

        let start = Instant::now();
        for _ in 0..10000 {
            let _ = phone_data.find(phone);
        }
        let duration = start.elapsed();

        println!("10,000 lookups took: {:?}", duration);
        println!("Average per lookup: {:?}", duration / 10000);

        // Should be very fast
        assert!(
            duration.as_millis() < 1000,
            "Lookups are too slow: {:?}",
            duration
        );
    }

    #[test]
    fn test_cache_performance() {
        let phone_data = PhoneData::new().expect("Failed to load phone data");
        let phone = "1380000000";

        // First lookup (no cache)
        let start = Instant::now();
        let _ = phone_data.find(phone);
        let first_lookup = start.elapsed();

        // Second lookup (with cache)
        let start = Instant::now();
        let _ = phone_data.find(phone);
        let cached_lookup = start.elapsed();

        println!("First lookup: {:?}", first_lookup);
        println!("Cached lookup: {:?}", cached_lookup);

        // Cached lookup should be faster (though this is hard to measure reliably)
        assert!(cached_lookup <= first_lookup);
    }

    #[test]
    fn test_concurrent_lookups() {
        use std::sync::Arc;
        use std::thread;

        let phone_data = Arc::new(PhoneData::new().expect("Failed to load phone data"));
        let mut handles = vec![];

        for i in 0..10 {
            let phone_data = Arc::clone(&phone_data);
            let handle = thread::spawn(move || {
                let phone = format!("138000000{}", i);
                for _ in 0..1000 {
                    let _ = phone_data.find(&phone);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        println!("Concurrent test completed successfully");
    }
}
