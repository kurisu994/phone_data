// 公共类型和接口模块
pub mod common;

// 二分查找算法模块
pub mod binary_search;

// 导入各种查找算法模块
pub mod phone_hash;
pub mod phone_simd;
pub mod phone_bloom;

// 重新导出公共类型
pub use common::{PhoneNoInfo, ErrorKind, CardType, PhoneLookup, PhoneStats};

// 重新导出SIMD优化算法作为默认实现
pub use phone_simd::PhoneDataSimd as PhoneData;

// 便于从crate根导出其它实现类型
pub use phone_hash::PhoneDataHash;
pub use phone_bloom::PhoneDataBloom;
pub use phone_simd::PhoneDataSimd;