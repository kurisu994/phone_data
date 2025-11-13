use anyhow::Result;
use serde_derive::Serialize;

/// 公共错误类型
#[derive(Debug)]
pub enum ErrorKind {
    InvalidPhoneDatabase,
    InvalidLength,
    NotFound,
    InvalidOpNo,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ErrorKind::InvalidPhoneDatabase => write!(f, "invalid phone database."),
            ErrorKind::InvalidLength => write!(f, "length of phone number is invalid."),
            ErrorKind::NotFound => write!(f, "can not find this phone number in database."),
            ErrorKind::InvalidOpNo => write!(f, "invalid number to representative Communications Operators."),
        }
    }
}

impl std::error::Error for ErrorKind {}

/// 手机号查询结果结构体
#[derive(Debug, Serialize, Clone)]
pub struct PhoneNoInfo {
    /// 省份
    pub province: String,
    /// 城市
    pub city: String,
    /// 邮政编码
    pub zip_code: String,
    /// 长途区号
    pub area_code: String,
    /// 卡类型
    pub card_type: String,
}

impl PhoneNoInfo {
    /// 创建新的PhoneNoInfo实例
    pub fn new(
        province: String,
        city: String,
        zip_code: String,
        area_code: String,
        card_type: String,
    ) -> Self {
        Self {
            province,
            city,
            zip_code,
            area_code,
            card_type,
        }
    }
}

/// 运营商类型枚举
#[derive(Debug, Clone, Copy)]
pub enum CardType {
    Cmcc = 1,    // 中国移动
    Cucc = 2,    // 中国联通
    Ctcc = 3,    // 中国电信
    CtccV = 4,   // 中国电信虚拟运营商
    CuccV = 5,   // 中国联通虚拟运营商
    CmccV = 6,   // 中国移动虚拟运营商
    Cbcc = 7,    // 中国广电
    CbccV = 8,   // 中国广电虚拟运营商
}

impl CardType {
    /// 从字节转换为CardType枚举
    pub fn from_u8(i: u8) -> Result<CardType> {
        match i {
            1 => Ok(CardType::Cmcc),
            2 => Ok(CardType::Cucc),
            3 => Ok(CardType::Ctcc),
            4 => Ok(CardType::CtccV),
            5 => Ok(CardType::CuccV),
            6 => Ok(CardType::CmccV),
            7 => Ok(CardType::Cbcc),
            8 => Ok(CardType::CbccV),
            _ => Err(ErrorKind::InvalidOpNo.into()),
        }
    }

    /// 获取运营商描述
    pub fn get_description(&self) -> String {
        match self {
            CardType::Cmcc => "中国移动".to_string(),
            CardType::Cucc => "中国联通".to_string(),
            CardType::Ctcc => "中国电信".to_string(),
            CardType::CtccV => "中国电信虚拟运营商".to_string(),
            CardType::CuccV => "中国联通虚拟运营商".to_string(),
            CardType::CmccV => "中国移动虚拟运营商".to_string(),
            CardType::Cbcc => "中国广电".to_string(),
            CardType::CbccV => "中国广电虚拟运营商".to_string(),
        }
    }

    /// 获取运营商代码
    pub fn get_code(&self) -> u8 {
        *self as u8
    }
}

/// 索引结构体 - 用于二分查找等算法
#[derive(Debug, Serialize, Clone)]
pub struct Index {
    /// 手机号前七位
    pub phone_no_prefix: i32,
    /// 记录区的偏移
    pub records_offset: i32,
    /// 卡类型
    pub card_type: u8,
}

impl Index {
    pub fn new(phone_no_prefix: i32, records_offset: i32, card_type: u8) -> Self {
        Self {
            phone_no_prefix,
            records_offset,
            card_type,
        }
    }
}

/// 记录结构体 - 解析后的记录数据
#[derive(Debug, Clone)]
pub struct ParsedRecord {
    pub province: String,
    pub city: String,
    pub zip_code: String,
    pub area_code: String,
}

impl ParsedRecord {
    pub fn new(
        province: String,
        city: String,
        zip_code: String,
        area_code: String,
    ) -> Self {
        Self {
            province,
            city,
            zip_code,
            area_code,
        }
    }
}

/// 手机号查找器通用接口
pub trait PhoneLookup {
    /// 查找手机号信息
    fn find(&self, no: &str) -> Result<PhoneNoInfo>;

    /// 批量查找手机号信息
    fn find_batch(&self, phones: &[&str]) -> Vec<Result<PhoneNoInfo>> {
        phones.iter().map(|phone| self.find(phone)).collect()
    }

    /// 验证手机号格式
    fn validate_phone_no(&self, no: &str) -> Result<i32> {
        let len = no.len();
        if len < 7 || len > 11 {
            return Err(ErrorKind::InvalidLength.into());
        }

        // 解析前7位作为键
        if len == 7 {
            no.parse::<i32>()
        } else {
            no[..7].parse::<i32>()
        }.map_err(|_| ErrorKind::InvalidLength.into())
    }
}

/// 统计信息通用接口
pub trait PhoneStats {
    /// 获取记录总数
    fn total_entries(&self) -> usize;

    /// 获取版本信息
    fn version(&self) -> &str;

    /// 获取内存使用量（字节）
    fn memory_usage_bytes(&self) -> usize;
}

/// 数据库头部信息
#[derive(Debug, Clone)]
pub struct DatabaseHeader {
    pub version: String,
    pub index_offset: u64,
}

impl DatabaseHeader {
    pub fn new(version: String, index_offset: u64) -> Self {
        Self {
            version,
            index_offset,
        }
    }
}

/// 通用工具函数
pub mod utils {
    use super::*;

    /// 将4个字节转换为i32（小端序）
    pub fn four_u8_to_i32(s: &[u8]) -> i32 {
        if s.len() < 4 {
            return 0;
        }
        i32::from_le_bytes([s[0], s[1], s[2], s[3]])
    }

    /// 解析记录数据
    pub fn parse_record_data(records: &[u8], offset: usize) -> Result<ParsedRecord> {
        let record_end = match records[offset - 8..].iter().position(|&b| b == 0) {
            Some(pos) => offset - 8 + pos,
            None => return Err(ErrorKind::InvalidPhoneDatabase.into()),
        };

        let record_slice = &records[offset - 8..record_end];
        let record_str = std::str::from_utf8(record_slice)
            .map_err(|_| ErrorKind::InvalidPhoneDatabase)?;

        let mut parts = Vec::with_capacity(4);
        for part in record_str.split('|') {
            parts.push(part);
        }

        if parts.len() != 4 {
            return Err(ErrorKind::InvalidPhoneDatabase.into());
        }

        Ok(ParsedRecord {
            province: parts[0].to_string(),
            city: parts[1].to_string(),
            zip_code: parts[2].to_string(),
            area_code: parts[3].to_string(),
        })
    }

    /// 构建PhoneNoInfo
    pub fn build_phone_info(record: &ParsedRecord, card_type: u8) -> Result<PhoneNoInfo> {
        let card_type_enum = CardType::from_u8(card_type)?;
        Ok(PhoneNoInfo {
            province: record.province.clone(),
            city: record.city.clone(),
            zip_code: record.zip_code.clone(),
            area_code: record.area_code.clone(),
            card_type: card_type_enum.get_description(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_type_conversion() {
        let card_type = CardType::Cmcc;
        assert_eq!(card_type.get_code(), 1);
        assert_eq!(card_type.get_description(), "中国移动");
    }

    #[test]
    fn test_phone_validation() {
        // 这个测试需要具体的实现来提供
        // 这里只是一个示例
        assert!(true); // 占位符
    }

    #[test]
    fn test_utils_functions() {
        let test_bytes = [0x01, 0x02, 0x03, 0x04];
        let result = utils::four_u8_to_i32(&test_bytes);
        assert_eq!(result, 0x04030201);
    }
}