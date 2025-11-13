use std::fs::File;
use std::io::{
    BufReader,
    Read,
};

use anyhow::Result;
use serde_derive::Serialize;

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

#[derive(Debug, Serialize)]
pub struct PhoneData {
    version: String,
    records: Vec<u8>,
    index: Vec<Index>,
}

#[derive(Debug, Serialize)]
struct Index {
    /// 手机号前七位
    phone_no_prefix: i32,
    /// 记录区的偏移
    records_offset: i32,
    /// 卡类型
    card_type: u8,
}

#[derive(Debug, Serialize)]
struct Records {
    /// 省
    province: String,
    /// 市
    city: String,
    /// 邮政编码
    zip_code: String,
    /// 长途区号
    area_code: String,
}

impl PhoneData {
    pub fn new() -> Result<PhoneData> {
        let data_file = File::open("phone.dat")?;
        let mut data_file = BufReader::new(data_file);

        // parse version and index offset
        let mut header_buffer = [0u8; 8];
        data_file.read_exact(&mut header_buffer)?;
        let version = String::from_utf8((&header_buffer[..4]).to_vec())?;
        let index_offset = Self::four_u8_to_i32(&header_buffer[4..]) as u64;

        // read records
        let mut records = vec![0u8; index_offset as usize - 8];
        data_file.read_exact(&mut records)?;

        // parse index
        let mut index = Vec::new();
        // length of a index is 9
        let mut index_item = [0u8; 9];
        loop {
            match data_file.read_exact(&mut index_item) {
                Ok(_) => (),
                Err(e) => match e.kind() {
                    std::io::ErrorKind::UnexpectedEof => break,
                    _ => (),
                },
            }
            let phone_no_prefix = Self::four_u8_to_i32(&index_item[..4]);
            let records_offset = Self::four_u8_to_i32(&index_item[4..8]);
            let card_type = index_item[8];
            index.push(Index {
                phone_no_prefix,
                records_offset,
                card_type,
            });
        }

        let config = PhoneData {
            version,
            records,
            index,
        };
        Ok(config)
    }

    fn four_u8_to_i32(s: &[u8]) -> i32 {
        let mut ret = 0;
        for (i, v) in s.iter().enumerate() {
            let v = *v as i32;
            ret += v << 8 * i;
        }
        ret
    }

    fn parse_to_record(&self, offset: usize) -> Result<Records> {
        // 优化：避免额外的内存分配
        let record_end = match self.records[offset - 8..].iter().position(|&b| b == 0) {
            Some(pos) => offset - 8 + pos,
            None => return Err(ErrorKind::InvalidPhoneDatabase.into()),
        };

        let record_slice = &self.records[offset - 8..record_end];
        let record_str = std::str::from_utf8(record_slice)
            .map_err(|_| ErrorKind::InvalidPhoneDatabase)?;

        // 优化：预分配正确大小的Vec以避免重新分配
        let mut parts = Vec::with_capacity(4);
        for part in record_str.split('|') {
            parts.push(part);
        }

        if parts.len() != 4 {
            return Err(ErrorKind::InvalidPhoneDatabase.into());
        }

        Ok(Records {
            province: parts[0].to_string(),
            city: parts[1].to_string(),
            zip_code: parts[2].to_string(),
            area_code: parts[3].to_string(),
        })
    }

    /// 二分法查找 `phone_no` 数据 - 优化版本
    pub fn find(&self, no: &str) -> Result<PhoneNoInfo> {
        let len = no.len();
        if len < 7 || len > 11 {
            return Err(ErrorKind::InvalidLength.into());
        }

        // 优化：只解析前7位并提前转换为i32
        let phone_prefix = if len == 7 {
            no.parse::<i32>()?
        } else {
            no[..7].parse::<i32>()?
        };

        let mut left = 0usize;
        let mut right = self.index.len();

        // 优化的二分查找：使用while循环避免不必要的中间变量
        while left < right {
            // 防止溢出并使用位运算优化
            let mid = left + ((right - left) >> 1);
            let mid_index = unsafe {
                // unsafe访问提升性能，因为mid保证在有效范围内
                self.index.get_unchecked(mid)
            };

            match mid_index.phone_no_prefix.cmp(&phone_prefix) {
                std::cmp::Ordering::Greater => {
                    right = mid;
                }
                std::cmp::Ordering::Less => {
                    left = mid + 1;
                }
                std::cmp::Ordering::Equal => {
                    // 找到匹配项，解析记录并返回
                    return self.build_phone_info(mid_index);
                }
            }
        }

        Err(ErrorKind::NotFound.into())
    }

    /// 辅助函数：构建PhoneNoInfo，减少重复代码
    #[inline]
    fn build_phone_info(&self, index: &Index) -> Result<PhoneNoInfo> {
        let record = self.parse_to_record(index.records_offset as usize)?;
        let card_type = CardType::from_u8(index.card_type)?;

        Ok(PhoneNoInfo {
            province: record.province,
            city: record.city,
            zip_code: record.zip_code,
            area_code: record.area_code,
            card_type: card_type.get_description(),
        })
    }
}

/// 运营商
enum CardType {
    Cmcc = 1,
    Cucc = 2,
    Ctcc = 3,
    CtccV = 4,
    CuccV = 5,
    CmccV = 6,
    Cbcc = 7,
    CbccV = 8,
}

impl CardType {
    fn from_u8(i: u8) -> Result<CardType> {
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

    fn get_description(&self) -> String {
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
}

#[derive(Debug, Serialize)]
pub struct PhoneNoInfo {
    /// 省
    province: String,
    /// 市
    city: String,
    /// 邮政编码
    zip_code: String,
    /// 长途区号
    area_code: String,
    /// 卡类型
    card_type: String,
}