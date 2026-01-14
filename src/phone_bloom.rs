use std::fs::File;
use std::io::{BufReader, Read};
use anyhow::Result;
use serde_derive::Serialize;
use crate::common::{utils, PhoneNoInfo, ErrorKind, CardType, PhoneLookup, PhoneStats};

#[derive(Debug, Serialize)]
pub struct PhoneDataBloom {
    version: String,
    records: Vec<u8>,
    index: Vec<Index>,
    bloom_filter: BloomFilter,
}

#[derive(Debug, Serialize)]
struct Index {
    phone_no_prefix: i32,
    records_offset: i32,
    card_type: u8,
}

#[derive(Debug, Serialize)]
pub struct BloomFilter {
    bits: Vec<u64>,
    hash_count: u32,
    item_count: usize,
}

impl BloomFilter {
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        let bit_count = ((expected_items as f64) * false_positive_rate.ln() / (-2.0f64 * (2f64.ln()).powi(2))) as usize;
        let hash_count = ((bit_count as f64 / expected_items as f64) * 2f64.ln()) as u32;

        BloomFilter {
            bits: vec![0u64; (bit_count + 63) / 64],
            hash_count,
            item_count: 0,
        }
    }

    pub fn insert(&mut self, item: i32) {
        for i in 0..self.hash_count {
            let hash = self.hash(item, i);
            let bit_index = (hash % (self.bits.len() as u64 * 64)) as usize;
            let array_index = bit_index / 64;
            let bit_offset = bit_index % 64;
            self.bits[array_index] |= 1u64 << bit_offset;
        }
        self.item_count += 1;
    }

    pub fn contains(&self, item: i32) -> bool {
        for i in 0..self.hash_count {
            let hash = self.hash(item, i);
            let bit_index = (hash % (self.bits.len() as u64 * 64)) as usize;
            let array_index = bit_index / 64;
            let bit_offset = bit_index % 64;

            if (self.bits[array_index] & (1u64 << bit_offset)) == 0 {
                return false;
            }
        }
        true
    }

    fn hash(&self, item: i32, seed: u32) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish()
    }

    pub fn false_positive_rate(&self) -> f64 {
        if self.item_count == 0 {
            return 0.0;
        }

        let bits_per_item = (self.bits.len() * 64) as f64 / self.item_count as f64;
        (1.0 - (-1.0 / bits_per_item).exp()).powi(self.hash_count as i32)
    }
}

impl PhoneDataBloom {
    pub fn new() -> Result<PhoneDataBloom> {
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

        // 解析索引区并构建布隆过滤器
        let mut index = Vec::new();
        let mut index_item = [0u8; 9];
        let mut bloom_filter = BloomFilter::new(517258, 0.01); // 1% 误报率

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

            // 添加到布隆过滤器
            bloom_filter.insert(phone_no_prefix);

            index.push(Index {
                phone_no_prefix,
                records_offset,
                card_type,
            });
        }

        Ok(PhoneDataBloom {
            version,
            records,
            index,
            bloom_filter,
        })
    }

    /// 布隆过滤器优化的查找 - 先快速过滤，再精确查找
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

        // 快速布隆过滤器检查
        if !self.bloom_filter.contains(phone_prefix) {
            return Err(ErrorKind::NotFound.into());
        }

        // 布隆过滤器说可能存在，进行精确二分查找
        let result = self.binary_search(phone_prefix);

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

    /// 批量查找优化 - 利用布隆过滤器快速排除不存在的号码
    pub fn find_batch(&self, phones: &[&str]) -> Vec<Result<PhoneNoInfo>> {
        phones.iter().map(|phone| {
            let len = phone.len();
            if len < 7 || len > 11 {
                return Err(ErrorKind::InvalidLength.into());
            }

            let phone_prefix = if len == 7 {
                phone.parse::<i32>()?
            } else {
                phone[..7].parse::<i32>()?
            };

            // 快速布隆过滤器检查
            if !self.bloom_filter.contains(phone_prefix) {
                return Err(ErrorKind::NotFound.into());
            }

            // 精确查找
            match self.binary_search(phone_prefix) {
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
        }).collect()
    }

    /// 统计查找效率
    pub fn find_with_stats(&self, no: &str) -> (Result<PhoneNoInfo>, LookupStats) {
        let start = std::time::Instant::now();

        let len = no.len();
        if len < 7 || len > 11 {
            return (Err(ErrorKind::InvalidLength.into()), LookupStats {
                bloom_filter_time: start.elapsed(),
                binary_search_time: std::time::Duration::from_nanos(0),
                bloom_positive: false,
                found: false,
            });
        }

        let phone_prefix = if len == 7 {
            no.parse::<i32>().unwrap_or(0)
        } else {
            no[..7].parse::<i32>().unwrap_or(0)
        };

        // 布隆过滤器检查
        let bloom_start = std::time::Instant::now();
        let bloom_positive = self.bloom_filter.contains(phone_prefix);
        let bloom_time = bloom_start.elapsed();

        if !bloom_positive {
            return (Err(ErrorKind::NotFound.into()), LookupStats {
                bloom_filter_time: bloom_time,
                binary_search_time: std::time::Duration::from_nanos(0),
                bloom_positive: false,
                found: false,
            });
        }

        // 二分查找
        let binary_start = std::time::Instant::now();
        let result = match self.binary_search(phone_prefix) {
            Some(index) => {
                let record = utils::parse_record_data(&self.records, index.records_offset as usize).unwrap();
                let card_type = CardType::from_u8(index.card_type).unwrap();
                Ok(PhoneNoInfo {
                    province: record.province,
                    city: record.city,
                    zip_code: record.zip_code,
                    area_code: record.area_code,
                    card_type: card_type.get_description(),
                })
            }
            None => Err(ErrorKind::NotFound.into()),
        };
        let binary_time = binary_start.elapsed();
        let found = result.is_ok();

        (result, LookupStats {
            bloom_filter_time: bloom_time,
            binary_search_time: binary_time,
            bloom_positive,
            found,
        })
    }

    #[inline]
    fn binary_search(&self, target: i32) -> Option<&Index> {
        let mut left = 0usize;
        let mut right = self.index.len();

        while left < right {
            let mid = left + ((right - left) >> 1);
            let mid_index = unsafe { self.index.get_unchecked(mid) };

            match mid_index.phone_no_prefix.cmp(&target) {
                std::cmp::Ordering::Equal => return Some(mid_index),
                std::cmp::Ordering::Greater => right = mid,
                std::cmp::Ordering::Less => left = mid + 1,
            }
        }

        None
    }

    /// 获取统计信息
    pub fn stats(&self) -> BloomStats {
        BloomStats {
            total_entries: self.index.len(),
            version: self.version.clone(),
            bloom_filter_bits: self.bloom_filter.bits.len() * 64,
            bloom_filter_hash_count: self.bloom_filter.hash_count,
            estimated_false_positive_rate: self.bloom_filter.false_positive_rate(),
            memory_usage_bytes: self.records.len() +
                self.index.len() * std::mem::size_of::<Index>() +
                self.bloom_filter.bits.len() * std::mem::size_of::<u64>(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LookupStats {
    pub bloom_filter_time: std::time::Duration,
    pub binary_search_time: std::time::Duration,
    pub bloom_positive: bool,
    pub found: bool,
}

#[derive(Debug, Serialize)]
pub struct BloomStats {
    pub total_entries: usize,
    pub version: String,
    pub bloom_filter_bits: usize,
    pub bloom_filter_hash_count: u32,
    pub estimated_false_positive_rate: f64,
    pub memory_usage_bytes: usize,
}


impl PhoneLookup for PhoneDataBloom {
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

        // 快速布隆过滤器检查
        if !self.bloom_filter.contains(phone_prefix) {
            return Err(ErrorKind::NotFound.into());
        }

        // 布隆过滤器说可能存在，进行精确二分查找
        let result = self.binary_search(phone_prefix);

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

impl PhoneStats for PhoneDataBloom {
    fn total_entries(&self) -> usize {
        self.index.len()
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn memory_usage_bytes(&self) -> usize {
        self.records.len() +
        self.index.len() * std::mem::size_of::<Index>() +
        self.bloom_filter.bits.len() * std::mem::size_of::<u64>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_lookup() {
        let phone_data = PhoneDataBloom::new().unwrap();
        let result = phone_data.find("18086834111").unwrap();
        assert!(!result.province.is_empty());
        assert!(!result.city.is_empty());
        assert!(!result.card_type.is_empty());
    }

    #[test]
    fn test_bloom_filter_negative() {
        let phone_data = PhoneDataBloom::new().unwrap();
        let result = phone_data.find("99999999999");
        assert!(result.is_err());
    }
}