use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use anyhow::Result;
use serde_derive::Serialize;
use crate::common::{PhoneNoInfo, ErrorKind, CardType, PhoneLookup, PhoneStats};

#[derive(Debug, Serialize)]
pub struct PhoneDataHash {
    version: String,
    // 使用HashMap存储手机号前缀到记录的映射
    phone_map: HashMap<i32, PhoneRecord>,
}

#[derive(Debug, Serialize, Clone)]
struct PhoneRecord {
    province: String,
    city: String,
    zip_code: String,
    area_code: String,
    card_type: u8,
}



impl PhoneDataHash {
    /// 创建新的哈希版本手机数据实例
    pub fn new() -> Result<PhoneDataHash> {
        let data_file = File::open("phone.dat")?;
        let mut data_file = BufReader::new(data_file);

        // 解析版本号和索引偏移
        let mut header_buffer = [0u8; 8];
        data_file.read_exact(&mut header_buffer)?;
        let version = String::from_utf8((&header_buffer[..4]).to_vec())?;
        let index_offset = Self::four_u8_to_i32(&header_buffer[4..]) as u64;

        // 读取记录区
        let mut records = vec![0u8; index_offset as usize - 8];
        data_file.read_exact(&mut records)?;

        // 解析索引区并构建哈希表
        let mut phone_map = HashMap::with_capacity(517258); // 预分配容量
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
            let records_offset = Self::four_u8_to_i32(&index_item[4..]);
            let card_type = index_item[8];

            // 解析记录
            let record = Self::parse_to_record(&records, records_offset as usize)?;

            // 插入到哈希表
            phone_map.insert(phone_no_prefix, PhoneRecord {
                province: record.province,
                city: record.city,
                zip_code: record.zip_code,
                area_code: record.area_code,
                card_type,
            });
        }

        Ok(PhoneDataHash {
            version,
            phone_map,
        })
    }

    /// 使用哈希表查找手机号信息 - O(1) 平均时间复杂度
    pub fn find(&self, no: &str) -> Result<PhoneNoInfo> {
        let len = no.len();
        if len < 7 || len > 11 {
            return Err(ErrorKind::InvalidLength.into());
        }

        // 解析前7位作为键
        let phone_prefix = if len == 7 {
            no.parse::<i32>()?
        } else {
            no[..7].parse::<i32>()?
        };

        // 哈希表查找
        match self.phone_map.get(&phone_prefix) {
            Some(record) => {
                let card_type = CardType::from_u8(record.card_type)?;
                Ok(PhoneNoInfo {
                    province: record.province.clone(),
                    city: record.city.clone(),
                    zip_code: record.zip_code.clone(),
                    area_code: record.area_code.clone(),
                    card_type: card_type.get_description(),
                })
            }
            None => Err(ErrorKind::NotFound.into()),
        }
    }

    /// 获取哈希表统计信息
    pub fn stats(&self) -> HashMapStats {
        HashMapStats {
            total_entries: self.phone_map.len(),
            version: self.version.clone(),
        }
    }

    fn four_u8_to_i32(s: &[u8]) -> i32 {
        if s.len() < 4 {
            return 0;
        }
        i32::from_le_bytes([s[0], s[1], s[2], s[3]])
    }

    fn parse_to_record(records: &[u8], offset: usize) -> Result<ParsedRecord> {
        // 找到记录结束位置（遇到0字节）
        let record_end = match records[offset - 8..].iter().position(|&b| b == 0) {
            Some(pos) => offset - 8 + pos,
            None => return Err(ErrorKind::InvalidPhoneDatabase.into()),
        };

        let record_slice = &records[offset - 8..record_end];
        let record_str = std::str::from_utf8(record_slice)
            .map_err(|_| ErrorKind::InvalidPhoneDatabase)?;

        // 解析记录字段
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
}

#[derive(Debug)]
struct ParsedRecord {
    province: String,
    city: String,
    zip_code: String,
    area_code: String,
}

#[derive(Debug, Serialize)]
pub struct HashMapStats {
    pub total_entries: usize,
    pub version: String,
}

impl PhoneLookup for PhoneDataHash {
    fn find(&self, no: &str) -> Result<PhoneNoInfo> {
        let len = no.len();
        if len < 7 || len > 11 {
            return Err(ErrorKind::InvalidLength.into());
        }

        // 解析前7位作为键
        let phone_prefix = if len == 7 {
            no.parse::<i32>()?
        } else {
            no[..7].parse::<i32>()?
        };

        // HashMap查找
        match self.phone_map.get(&phone_prefix) {
            Some(record) => {
                let card_type = CardType::from_u8(record.card_type)?;
                Ok(PhoneNoInfo {
                    province: record.province.clone(),
                    city: record.city.clone(),
                    zip_code: record.zip_code.clone(),
                    area_code: record.area_code.clone(),
                    card_type: card_type.get_description(),
                })
            }
            None => Err(ErrorKind::NotFound.into()),
        }
    }
}

impl PhoneStats for PhoneDataHash {
    fn total_entries(&self) -> usize {
        self.phone_map.len()
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn memory_usage_bytes(&self) -> usize {
        std::mem::size_of::<Self>() +
        self.phone_map.capacity() * std::mem::size_of::<(i32, PhoneRecord)>() +
        self.phone_map.len() * std::mem::size_of::<PhoneRecord>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_lookup() {
        let phone_data = PhoneDataHash::new().unwrap();
        let result = phone_data.find("18086834111").unwrap();
        // 验证能正常查找，不关心具体省份
        assert!(!result.province.is_empty());
        assert!(!result.city.is_empty());
        assert!(!result.card_type.is_empty());
    }
}