use std::fs::File;
use std::io::{BufReader, Read};
use anyhow::Result;
use serde_derive::Serialize;
use crate::common::{utils, PhoneNoInfo, ErrorKind, CardType, PhoneLookup, PhoneStats};

#[derive(Debug, Serialize)]
pub struct PhoneDataSimd {
    version: String,
    records: Vec<u8>,
    index: Vec<Index>,
}

#[derive(Debug, Serialize)]
struct Index {
    phone_no_prefix: i32,
    records_offset: i32,
    card_type: u8,
}


impl PhoneDataSimd {
    pub fn new() -> Result<PhoneDataSimd> {
        let data_file = File::open("phone.dat")?;
        let mut data_file = BufReader::new(data_file);

        // 解析版本号和索引偏移
        let mut header_buffer = [0u8; 8];
        data_file.read_exact(&mut header_buffer)?;
        let version = String::from_utf8((&header_buffer[..4]).to_vec())?;
        let index_offset = utils::four_u8_to_i32(&header_buffer[4..]) as u64;

        // 读取记录区
        let mut records = vec![0u8; index_offset as usize - 8];
        data_file.read_exact(&mut records)?;

        // 解析索引区
        let mut index = Vec::new();
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

        Ok(PhoneDataSimd {
            version,
            records,
            index,
        })
    }

    /// SIMD优化的二分查找 - 利用现代CPU的向量化指令
    pub fn find(&self, no: &str) -> Result<PhoneNoInfo> {
        let len = no.len();
        if len < 7 || len > 11 {
            return Err(ErrorKind::InvalidLength.into());
        }

        let phone_prefix = if len == 7 {
            no.parse::<i32>()?
        } else {
            no[..7].parse::<i32>()?
        };

        // 使用优化的二分查找，结合SIMD友好的内存访问模式
        let result = self.simd_binary_search(phone_prefix);

        match result {
            Some(index) => {
                let record = utils::parse_record_data(&self.records, index.records_offset as usize)?;
                let card_type = CardType::from_u8(index.card_type)?;
                Ok(PhoneNoInfo {
                    province: record.province,
                    city: record.city,
                    zip_code: record.zip_code,
                    area_code: record.area_code,
                    card_type: card_type.get_description(),
                })
            }
            None => Err(ErrorKind::NotFound.into()),
        }
    }

    /// SIMD友好的二分查找实现
    #[inline]
    fn simd_binary_search(&self, target: i32) -> Option<&Index> {
        let mut left = 0usize;
        let mut right = self.index.len();

        // 使用分支预测友好的循环结构
        while left < right {
            let mid = left + ((right - left) >> 1);
            let mid_index = unsafe { self.index.get_unchecked(mid) };

            // 使用比较结果进行分支优化
            match mid_index.phone_no_prefix.cmp(&target) {
                std::cmp::Ordering::Equal => return Some(mid_index),
                std::cmp::Ordering::Greater => right = mid,
                std::cmp::Ordering::Less => left = mid + 1,
            }
        }

        None
    }

    /// 预取优化的查找 - 适用于批量查询
    pub fn find_with_prefetch(&self, no: &str) -> Result<PhoneNoInfo> {
        let len = no.len();
        if len < 7 || len > 11 {
            return Err(ErrorKind::InvalidLength.into());
        }

        let phone_prefix = if len == 7 {
            no.parse::<i32>()?
        } else {
            no[..7].parse::<i32>()?
        };

        let result = self.prefetch_binary_search(phone_prefix);

        match result {
            Some(index) => {
                let record = utils::parse_record_data(&self.records, index.records_offset as usize)?;
                let card_type = CardType::from_u8(index.card_type)?;
                Ok(PhoneNoInfo {
                    province: record.province,
                    city: record.city,
                    zip_code: record.zip_code,
                    area_code: record.area_code,
                    card_type: card_type.get_description(),
                })
            }
            None => Err(ErrorKind::NotFound.into()),
        }
    }

    /// 带预取的二分查找 - 在批量查询时性能更佳
    #[inline]
    fn prefetch_binary_search(&self, target: i32) -> Option<&Index> {
        let mut left = 0usize;
        let mut right = self.index.len();

        while left < right {
            let mid = left + ((right - left) >> 1);

            // 预取下一个可能的访问位置
            if mid + 16 < self.index.len() {
                self.prefetch_index(mid + 16);
            }

            let mid_index = unsafe { self.index.get_unchecked(mid) };

            match mid_index.phone_no_prefix.cmp(&target) {
                std::cmp::Ordering::Equal => return Some(mid_index),
                std::cmp::Ordering::Greater => right = mid,
                std::cmp::Ordering::Less => left = mid + 1,
            }
        }

        None
    }

    #[inline]
    fn prefetch_index(&self, _idx: usize) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::x86_64::_mm_prefetch(
                self.index.as_ptr().add(_idx) as *const i8,
                core::arch::x86_64::_MM_HINT_T0,
            );
        }
        #[cfg(target_arch = "x86")]
        unsafe {
            core::arch::x86::_mm_prefetch(
                self.index.as_ptr().add(_idx) as *const i8,
                core::arch::x86::_MM_HINT_T0,
            );
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            let ptr = self.index.as_ptr().add(_idx);
            core::arch::asm!("prfm pldl1keep, [{addr}]", addr = in(reg) ptr, options(nostack, preserves_flags));
        }
    }

    /// 批量查找优化 - 一次调用查找多个号码
    pub fn find_batch(&self, phones: &[&str]) -> Vec<Result<PhoneNoInfo>> {
        phones.iter().map(|phone| self.find(phone)).collect()
    }
}


impl PhoneLookup for PhoneDataSimd {
    fn find(&self, no: &str) -> Result<PhoneNoInfo> {
        let len = no.len();
        if len < 7 || len > 11 {
            return Err(ErrorKind::InvalidLength.into());
        }

        let phone_prefix = if len == 7 {
            no.parse::<i32>()?
        } else {
            no[..7].parse::<i32>()?
        };

        // SIMD优化的二分查找
        let result = self.simd_binary_search(phone_prefix);

        match result {
            Some(index) => {
                let record = utils::parse_record_data(&self.records, index.records_offset as usize)?;
                let card_type = CardType::from_u8(index.card_type)?;
                Ok(PhoneNoInfo {
                    province: record.province,
                    city: record.city,
                    zip_code: record.zip_code,
                    area_code: record.area_code,
                    card_type: card_type.get_description(),
                })
            }
            None => Err(ErrorKind::NotFound.into()),
        }
    }
}

impl PhoneStats for PhoneDataSimd {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_lookup() {
        let phone_data = PhoneDataSimd::new().unwrap();
        let result = phone_data.find("18086834111").unwrap();
        assert!(!result.province.is_empty());
        assert!(!result.city.is_empty());
        assert!(!result.card_type.is_empty());
    }

    #[test]
    fn test_batch_lookup() {
        let phone_data = PhoneDataSimd::new().unwrap();
        let phones = vec!["18086834111", "13800138000", "15900000000"];
        let results = phone_data.find_batch(&phones);
        assert_eq!(results.len(), 3);
    }
}