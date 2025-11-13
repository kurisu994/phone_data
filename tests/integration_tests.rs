use phone_data::{PhoneData, PhoneLookup, PhoneDataHash, PhoneDataSimd, PhoneDataBloom};
use phone_data::common::{PhoneStats, ErrorKind};

/// 集成测试模块 - 测试所有算法实现的兼容性
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_all_algorithms_compatibility() {
        // 测试所有算法实现都能正常工作
        let binary_data = PhoneData::new().expect("Failed to create binary search data");
        let hash_data = PhoneDataHash::new().expect("Failed to create hash data");
        let simd_data = PhoneDataSimd::new().expect("Failed to create SIMD data");
        let bloom_data = PhoneDataBloom::new().expect("Failed to create bloom data");

        // 使用相同的测试用例验证所有实现
        let test_phones = vec![
            "18086834111",
            "13800138000",
            "15900000000",
            "18612345678",
            "13344445555",
        ];

        for phone in test_phones {
            let binary_result = binary_data.find(phone);
            let hash_result = hash_data.find(phone);
            let simd_result = simd_data.find(phone);
            let bloom_result = bloom_data.find(phone);

            // 所有实现应该返回相同的结果
            assert!(binary_result.is_ok(), "Binary search failed for {}", phone);
            assert!(hash_result.is_ok(), "Hash lookup failed for {}", phone);
            assert!(simd_result.is_ok(), "SIMD lookup failed for {}", phone);
            assert!(bloom_result.is_ok(), "Bloom lookup failed for {}", phone);

            // 验证结果的省份字段不为空
            let binary_info = binary_result.unwrap();
            let hash_info = hash_result.unwrap();
            let simd_info = simd_result.unwrap();
            let bloom_info = bloom_result.unwrap();

            assert!(!binary_info.province.is_empty(), "Binary search returned empty province");
            assert!(!hash_info.province.is_empty(), "Hash lookup returned empty province");
            assert!(!simd_info.province.is_empty(), "SIMD lookup returned empty province");
            assert!(!bloom_info.province.is_empty(), "Bloom lookup returned empty province");
        }
    }

    #[test]
    fn test_failed_lookups_consistency() {
        // 测试失败的查找在所有实现中的一致行为
        let binary_data = PhoneData::new().expect("Failed to create binary search data");
        let hash_data = PhoneDataHash::new().expect("Failed to create hash data");
        let simd_data = PhoneDataSimd::new().expect("Failed to create SIMD data");
        let bloom_data = PhoneDataBloom::new().expect("Failed to create bloom data");

        let invalid_phones = vec![
            "99999999999",  // 不存在的号段
            "12345678901",  // 不存在的号段
            "11111111111",  // 不存在的号段
            "00000000000",  // 不存在的号段
            "1",             // 太短
            "123456789012",  // 太长
        ];

        for phone in invalid_phones {
            let binary_result = binary_data.find(phone);
            let hash_result = hash_data.find(phone);
            let simd_result = simd_data.find(phone);
            let bloom_result = bloom_data.find(phone);

            // 所有实现都应该失败
            assert!(binary_result.is_err(), "Binary search should fail for {}", phone);
            assert!(hash_result.is_err(), "Hash lookup should fail for {}", phone);
            assert!(simd_result.is_err(), "SIMD lookup should fail for {}", phone);
            assert!(bloom_result.is_err(), "Bloom lookup should fail for {}", phone);

            // 验证错误类型一致
            assert!(matches!(binary_result.err().unwrap().downcast_ref::<ErrorKind>(), &ErrorKind::NotFound));
            assert!(matches!(hash_result.err().unwrap().downcast_ref::<ErrorKind>(), &ErrorKind::NotFound));
            assert!(matches!(simd_result.err().unwrap().downcast_ref::<ErrorKind>(), &ErrorKind::NotFound));
            // 布隆过滤器可能提前过滤，但也应该返回NotFound
            assert!(matches!(bloom_result.err().unwrap().downcast_ref::<ErrorKind>(), &ErrorKind::NotFound));
        }
    }

    #[test]
    fn test_batch_lookup_consistency() {
        // 测试批量查找的一致性
        let binary_data = PhoneData::new().expect("Failed to create binary search data");
        let hash_data = PhoneDataHash::new().expect("Failed to create hash data");
        let simd_data = PhoneDataSimd::new().expect("Failed to create SIMD data");
        let bloom_data = PhoneDataBloom::new().expect("Failed to create bloom data");

        let test_phones = vec!["18086834111", "13800138000", "15900000000"];

        let binary_results = binary_data.find_batch(&test_phones);
        let hash_results = hash_data.find_batch(&test_phones);
        let simd_results = simd_data.find_batch(&test_phones);
        let bloom_results = bloom_data.find_batch(&test_phones);

        assert_eq!(binary_results.len(), hash_results.len());
        assert_eq!(binary_results.len(), simd_results.len());
        assert_eq!(binary_results.len(), bloom_results.len());
        assert_eq!(binary_results.len(), test_phones.len());

        for (i, phone) in test_phones.iter().enumerate() {
            assert!(binary_results[i].is_ok(), "Binary search batch failed for {}", phone);
            assert!(hash_results[i].is_ok(), "Hash lookup batch failed for {}", phone);
            assert!(simd_results[i].is_ok(), "SIMD lookup batch failed for {}", phone);
            assert!(bloom_results[i].is_ok(), "Bloom lookup batch failed for {}", phone);
        }
    }

    #[test]
    fn test_stats_consistency() {
        // 测试统计信息的一致性
        let binary_data = PhoneData::new().expect("Failed to create binary search data");
        let hash_data = PhoneDataHash::new().expect("Failed to create hash data");
        let simd_data = PhoneDataSimd::new().expect("Failed to create SIMD data");
        let bloom_data = PhoneDataBloom::new().expect("Failed to create bloom data");

        // 所有实现应该有相同的记录数和版本
        assert_eq!(binary_data.total_entries(), hash_data.total_entries());
        assert_eq!(binary_data.total_entries(), simd_data.total_entries());
        assert_eq!(binary_data.total_entries(), bloom_data.total_entries());

        assert_eq!(binary_data.version(), hash_data.version());
        assert_eq!(binary_data.version(), simd_data.version());
        assert_eq!(binary_data.version(), bloom_data.version());

        // 内存使用量应该相同（对于相同的数据结构）
        assert_eq!(binary_data.memory_usage_bytes(), simd_data.memory_usage_bytes());

        // 哈希和布隆过滤器会有额外的内存开销
        assert!(hash_data.memory_usage_bytes() > binary_data.memory_usage_bytes());
        assert!(bloom_data.memory_usage_bytes() > binary_data.memory_usage_bytes());
    }

    #[test]
    fn test_edge_cases() {
        let binary_data = PhoneData::new().expect("Failed to create binary search data");

        // 测试7位手机号
        let result = binary_data.find("1808683");
        assert!(result.is_ok(), "7-digit phone number should work");

        // 测试11位手机号
        let result = binary_data.find("18086834111");
        assert!(result.is_ok(), "11-digit phone number should work");

        // 测试边界情况
        assert!(binary_data.find("").is_err());
        assert!(binary_data.find("123456").is_err());
        assert!(binary_data.find("123456789012").is_err());
    }

    #[test]
    fn test_operator_types() {
        let binary_data = PhoneData::new().expect("Failed to create binary search data");

        // 测试不同运营商的手机号段
        let test_cases = vec![
            ("18086834111", "移动"),  // 中国移动
            ("18612345678", "移动"),  // 中国移动
            ("13344445555", "联通"),  // 中国联通
            ("17766668888", "联通"),  // 中国联通
            ("18999987777", "电信"),  // 中国电信
            ("19988887777", "电信"),  // 中国电信
        ];

        for (phone, expected_operator) in test_cases {
            let result = binary_data.find(phone).expect("Failed to lookup phone");
            assert!(result.card_type.contains(expected_operator),
                "Operator mismatch for {}. Expected: {}, Got: {}",
                phone, expected_operator, result.card_type);
        }
    }

    #[test]
    fn test_data_integrity() {
        let binary_data = PhoneData::new().expect("Failed to create binary search data");

        // 验证数据完整性
        assert!(!binary_data.version().is_empty(), "Version should not be empty");
        assert!(binary_data.total_entries() > 500000, "Should have significant number of records");
        assert!(binary_data.memory_usage_bytes() > 1000000, "Should use reasonable amount of memory");

        // 测试一些已知的手机号段
        let known_phones = vec![
            "13800138000", // 中国移动
            "18612345678", // 中国移动
            "13344445555", // 中国联通
            "17766668888", // 中国联通
            "18999987777", // 中国电信
        ];

        for phone in known_phones {
            assert!(binary_data.find(phone).is_ok(),
                "Known phone number {} should be found", phone);
        }
    }
}