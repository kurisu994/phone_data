use std::fs::File;
use std::io::{BufReader, Read};
use anyhow::Result;
use serde_derive::Serialize;
use crate::common::{utils, Index, ParsedRecord, PhoneNoInfo, PhoneLookup, PhoneStats, ErrorKind};



#[derive(Debug, Serialize)]
pub struct PhoneData {
    version: String,
    records: Vec<u8>,
    index: Vec<Index>,
}


impl PhoneData {
    pub fn new() -> Result<PhoneData> {
        let data_file = File::open("phone.dat")?;
        let mut data_file = BufReader::new(data_file);

        // parse version and index offset
        let mut header_buffer = [0u8; 8];
        data_file.read_exact(&mut header_buffer)?;
        let version = String::from_utf8((&header_buffer[..4]).to_vec())?;
        let index_offset = utils::four_u8_to_i32(&header_buffer[4..]) as u64;

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
            let phone_no_prefix = utils::four_u8_to_i32(&index_item[..4]);
            let records_offset = utils::four_u8_to_i32(&index_item[4..8]);
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

  
    fn parse_to_record(&self, offset: usize) -> Result<ParsedRecord> {
        crate::common::utils::parse_record_data(&self.records, offset)
    }

    
    /// 辅助函数：构建PhoneNoInfo，减少重复代码
    #[inline]
    fn build_phone_info(&self, index: &Index) -> Result<PhoneNoInfo> {
        let record = self.parse_to_record(index.records_offset as usize)?;
        crate::common::utils::build_phone_info(&record, index.card_type)
    }
}

impl PhoneLookup for PhoneData {
    fn find(&self, no: &str) -> Result<PhoneNoInfo> {
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
}

impl PhoneStats for PhoneData {
    fn total_entries(&self) -> usize {
        self.index.len()
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn memory_usage_bytes(&self) -> usize {
        self.records.len() + self.index.len() * std::mem::size_of::<Index>()
    }
}

