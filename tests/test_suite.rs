use phone_data::{PhoneData, PhoneLookup};
use phone_data::phone_hash::PhoneDataHash;
use phone_data::phone_simd::PhoneDataSimd;
use phone_data::phone_bloom::PhoneDataBloom;
use phone_data::common::PhoneStats;

/// 测试所有算法实现的基本功能
pub fn test_basic_functionality() {
    println!("测试基本功能...");

    let test_cases = vec![
        ("18086834111", "电信"),  // 四川成都-中国电信
        ("13800138000", "移动"),  // 北京-中国移动
    ];

    // 测试二分查找
    let binary_data = PhoneData::new().expect("Failed to create binary search data");
    for (phone, expected_operator) in &test_cases {
        let result = binary_data.find(phone);
        assert!(result.is_ok(), "二分查找失败: {}", phone);
        let info = result.unwrap();
        assert!(info.card_type.contains(expected_operator),
                "运营商不匹配: {}. 期望: {}, 实际: {}",
                phone, expected_operator, info.card_type);
    }
    println!("✓ 二分查找基本功能测试通过");

    // 测试哈希查找
    let hash_data = PhoneDataHash::new().expect("Failed to create hash data");
    for (phone, expected_operator) in &test_cases {
        let result = hash_data.find(phone);
        assert!(result.is_ok(), "哈希查找失败: {}", phone);
        let info = result.unwrap();
        assert!(info.card_type.contains(expected_operator),
                "运营商不匹配: {}. 期望: {}, 实际: {}",
                phone, expected_operator, info.card_type);
    }
    println!("✓ 哈希查找基本功能测试通过");

    // 测试SIMD查找
    let simd_data = PhoneDataSimd::new().expect("Failed to create SIMD data");
    for (phone, expected_operator) in &test_cases {
        let result = simd_data.find(phone);
        assert!(result.is_ok(), "SIMD查找失败: {}", phone);
        let info = result.unwrap();
        assert!(info.card_type.contains(expected_operator),
                "运营商不匹配: {}. 期望: {}, 实际: {}",
                phone, expected_operator, info.card_type);
    }
    println!("✓ SIMD查找基本功能测试通过");

    // 测试布隆过滤器查找
    let bloom_data = PhoneDataBloom::new().expect("Failed to create bloom data");
    for (phone, expected_operator) in &test_cases {
        let result = bloom_data.find(phone);
        assert!(result.is_ok(), "布隆过滤器查找失败: {}", phone);
        let info = result.unwrap();
        assert!(info.card_type.contains(expected_operator),
                "运营商不匹配: {}. 期望: {}, 实际: {}",
                phone, expected_operator, info.card_type);
    }
    println!("✓ 布隆过滤器查找基本功能测试通过");
}

/// 测试算法一致性
pub fn test_algorithm_consistency() {
    println!("测试算法一致性...");

    let binary_data = PhoneData::new().expect("Failed to create binary search data");
    let hash_data = PhoneDataHash::new().expect("Failed to create hash data");
    let simd_data = PhoneDataSimd::new().expect("Failed to create SIMD data");
    let bloom_data = PhoneDataBloom::new().expect("Failed to create bloom data");

    let test_phones = vec![
        "18086834111",
        "13800138000",
    ];

    for phone in test_phones {
        let binary_result = binary_data.find(phone);
        let hash_result = hash_data.find(phone);
        let simd_result = simd_data.find(phone);
        let bloom_result = bloom_data.find(phone);

        // 所有算法都应该成功
        assert!(binary_result.is_ok(), "二分查找失败: {}", phone);
        assert!(hash_result.is_ok(), "哈希查找失败: {}", phone);
        assert!(simd_result.is_ok(), "SIMD查找失败: {}", phone);
        assert!(bloom_result.is_ok(), "布隆过滤器查找失败: {}", phone);

        // 验证结果一致性
        let binary_info = binary_result.unwrap();
        let hash_info = hash_result.unwrap();
        let simd_info = simd_result.unwrap();
        let bloom_info = bloom_result.unwrap();

        assert_eq!(binary_info.province, hash_info.province, "省份不一致: {}", phone);
        assert_eq!(binary_info.city, hash_info.city, "城市不一致: {}", phone);
        assert_eq!(binary_info.card_type, hash_info.card_type, "运营商不一致: {}", phone);

        assert_eq!(binary_info.province, simd_info.province, "省份不一致: {}", phone);
        assert_eq!(binary_info.city, simd_info.city, "城市不一致: {}", phone);
        assert_eq!(binary_info.card_type, simd_info.card_type, "运营商不一致: {}", phone);

        assert_eq!(binary_info.province, bloom_info.province, "省份不一致: {}", phone);
        assert_eq!(binary_info.city, bloom_info.city, "城市不一致: {}", phone);
        assert_eq!(binary_info.card_type, bloom_info.card_type, "运营商不一致: {}", phone);
    }
    println!("✓ 算法一致性测试通过");
}

/// 测试错误处理
pub fn test_error_handling() {
    println!("测试错误处理...");

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

        // 所有算法都应该失败
        assert!(binary_result.is_err(), "二分查找应该失败: {}", phone);
        assert!(hash_result.is_err(), "哈希查找应该失败: {}", phone);
        assert!(simd_result.is_err(), "SIMD查找应该失败: {}", phone);
        assert!(bloom_result.is_err(), "布隆过滤器查找应该失败: {}", phone);
    }
    println!("✓ 错误处理测试通过");
}

/// 测试批量查找
pub fn test_batch_lookup() {
    println!("测试批量查找...");

    let binary_data = PhoneData::new().expect("Failed to create binary search data");
    let hash_data = PhoneDataHash::new().expect("Failed to create hash data");
    let simd_data = PhoneDataSimd::new().expect("Failed to create SIMD data");
    let bloom_data = PhoneDataBloom::new().expect("Failed to create bloom data");

    let test_phones = vec![
        "18086834111",
        "13800138000",
    ];

    let binary_results = binary_data.find_batch(&test_phones);
    let hash_results = hash_data.find_batch(&test_phones);
    let simd_results = simd_data.find_batch(&test_phones);
    let bloom_results = bloom_data.find_batch(&test_phones);

    assert_eq!(binary_results.len(), hash_results.len());
    assert_eq!(binary_results.len(), simd_results.len());
    assert_eq!(binary_results.len(), bloom_results.len());
    assert_eq!(binary_results.len(), test_phones.len());

    for (i, phone) in test_phones.iter().enumerate() {
        assert!(binary_results[i].is_ok(), "二分查找批量失败: {}", phone);
        assert!(hash_results[i].is_ok(), "哈希查找批量失败: {}", phone);
        assert!(simd_results[i].is_ok(), "SIMD查找批量失败: {}", phone);
        assert!(bloom_results[i].is_ok(), "布隆过滤器查找批量失败: {}", phone);
    }
    println!("✓ 批量查找测试通过");
}

/// 测试统计信息
pub fn test_statistics() {
    println!("测试统计信息...");

    let binary_data = PhoneData::new().expect("Failed to create binary search data");
    let hash_data = PhoneDataHash::new().expect("Failed to create hash data");
    let simd_data = PhoneDataSimd::new().expect("Failed to create SIMD data");
    let bloom_data = PhoneDataBloom::new().expect("Failed to create bloom data");

    // 验证记录数一致
    assert_eq!(binary_data.total_entries(), hash_data.total_entries());
    assert_eq!(binary_data.total_entries(), simd_data.total_entries());
    assert_eq!(binary_data.total_entries(), bloom_data.total_entries());

    // 验证版本一致
    assert_eq!(binary_data.version(), hash_data.version());
    assert_eq!(binary_data.version(), simd_data.version());
    assert_eq!(binary_data.version(), bloom_data.version());

    println!("总记录数: {}", binary_data.total_entries());
    println!("版本: {}", binary_data.version());
    println!("二分查找内存使用: {} bytes", binary_data.memory_usage_bytes());
    println!("哈希查找内存使用: {} bytes", hash_data.memory_usage_bytes());
    println!("SIMD查找内存使用: {} bytes", simd_data.memory_usage_bytes());
    println!("布隆过滤器内存使用: {} bytes", bloom_data.memory_usage_bytes());

    // 验证内存使用合理
    assert!(binary_data.memory_usage_bytes() > 1000000, "内存使用应该超过1MB");
    assert!(hash_data.memory_usage_bytes() > binary_data.memory_usage_bytes(),
            "哈希查找内存使用应该大于二分查找");
    assert!(bloom_data.memory_usage_bytes() > binary_data.memory_usage_bytes(),
            "布隆过滤器内存使用应该大于二分查找");

    println!("✓ 统计信息测试通过");
}

/// 测试边界情况
pub fn test_edge_cases() {
    println!("测试边界情况...");

    let binary_data = PhoneData::new().expect("Failed to create binary search data");

    // 测试7位手机号
    let result = binary_data.find("1808683");
    assert!(result.is_ok(), "7位手机号应该可以查询");
    println!("✓ 7位手机号查询测试通过");

    // 测试11位手机号
    let result = binary_data.find("18086834111");
    assert!(result.is_ok(), "11位手机号应该可以查询");
    println!("✓ 11位手机号查询测试通过");

    // 测试无效输入
    assert!(binary_data.find("").is_err());
    assert!(binary_data.find("123456").is_err());
    assert!(binary_data.find("123456789012").is_err());

    println!("✓ 边界情况测试通过");
}

/// 运行所有测试
pub fn run_all_tests() {
    println!("=== 手机号查询系统统一测试套件 ===\n");

    test_basic_functionality();
    println!();

    test_algorithm_consistency();
    println!();

    test_error_handling();
    println!();

    test_batch_lookup();
    println!();

    test_statistics();
    println!();

    test_edge_cases();
    println!();

    println!("=== 所有测试通过 ===");
}