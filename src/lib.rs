use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::Arc;
use std::sync::Mutex;
use std::collections::HashMap;

use serde::Serialize;
use thiserror::Error;

pub mod config;

#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("数据库文件格式无效")]
    InvalidPhoneDatabase,
    #[error("手机号码长度无效，有效长度为7-11位")]
    InvalidLength,
    #[error("在数据库中未找到此手机号码")]
    NotFound,
    #[error("无效的运营商代码")]
    InvalidOpNo,
    #[error("I/O 错误: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct PhoneData {
    version: String,
    records: Arc<Vec<u8>>,
    index: Arc<Vec<Index>>,
    // 添加简单的 LRU 缓存
    cache: Arc<Mutex<HashMap<String, PhoneNoInfo>>>,
    cache_enabled: bool,
    cache_max_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Index {
    /// 手机号前七位
    phone_no_prefix: i32,
    /// 记录区的偏移
    records_offset: i32,
    /// 卡类型
    card_type: u8,
}

impl PartialOrd for Index {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Index {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.phone_no_prefix.cmp(&other.phone_no_prefix)
    }
}

#[derive(Debug, Clone, Serialize)]
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
    /// 获取数据库版本信息
    pub fn version(&self) -> &str {
        &self.version
    }
    
    /// 获取索引记录数量
    pub fn index_count(&self) -> usize {
        self.index.len()
    }
    
    pub fn new() -> Fallible<PhoneData> {
        Self::from_file("phone.dat")
    }

    pub fn from_file(path: &str) -> Fallible<PhoneData> {
        Self::from_file_with_config(path, true, 1000)
    }

    pub fn from_file_with_config(path: &str, cache_enabled: bool, cache_max_size: usize) -> Fallible<PhoneData> {
        tracing::info!("正在加载手机号码数据库文件: {}", path);
        let data_file = File::open(path)?;
        let mut data_file = BufReader::new(data_file);

        // parse version and index offset
        let mut header_buffer = [0u8; 8];
        data_file
            .read_exact(&mut header_buffer)
            .map_err(|_| ErrorKind::InvalidPhoneDatabase)?;
        let version = String::from_utf8((&header_buffer[..4]).to_vec())
            .map_err(|_| ErrorKind::InvalidPhoneDatabase)?;
        let index_offset = Self::four_u8_to_i32(&header_buffer[4..]) as u64;

        // read records
        let mut records = vec![0u8; index_offset as usize - 8];
        data_file
            .read_exact(&mut records)
            .map_err(|_| ErrorKind::InvalidPhoneDatabase)?;

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
            version: version.clone(),
            records: Arc::new(records),
            index: Arc::new(index.clone()),
            cache: Arc::new(Mutex::new(HashMap::new())),
            cache_enabled,
            cache_max_size,
        };
        tracing::info!(
            "数据库加载完成，版本: {}, 索引数量: {}", 
            version, 
            index.len()
        );
        Ok(config)
    }

    #[inline]
    fn four_u8_to_i32(s: &[u8]) -> i32 {
        if s.len() >= 4 {
            i32::from_le_bytes([s[0], s[1], s[2], s[3]])
        } else {
            let mut bytes = [0u8; 4];
            bytes[..s.len()].copy_from_slice(s);
            i32::from_le_bytes(bytes)
        }
    }

    fn parse_to_record(&self, offset: usize) -> Fallible<Records> {
        if let Some(record) = self.records[offset - 8..].splitn(2, |i| *i == 0u8).nth(0) {
            let record =
                String::from_utf8(record.to_vec()).map_err(|_| ErrorKind::InvalidPhoneDatabase)?;
            let record: Vec<&str> = record.split('|').collect();
            if record.len() != 4 {
                return Err(ErrorKind::InvalidPhoneDatabase);
            }
            Ok(Records {
                province: record[0].to_string(),
                city: record[1].to_string(),
                zip_code: record[2].to_string(),
                area_code: record[3].to_string(),
            })
        } else {
            Err(ErrorKind::InvalidPhoneDatabase)
        }
    }

    /// 优化的二分查找算法查找 `phone_no` 数据
    pub fn find(&self, no: &str) -> Fallible<PhoneNoInfo> {
        let len = no.len();
        if !(7..=11).contains(&len) {
            return Err(ErrorKind::InvalidLength);
        }

        // 检查缓存（仅当缓存启用时）
        if self.cache_enabled {
            if let Ok(cache) = self.cache.lock() {
                if let Some(cached_result) = cache.get(no) {
                    tracing::debug!("从缓存返回手机号 {} 的信息", no);
                    return Ok(cached_result.clone());
                }
            }
        }

        // 快速解析前7位数字，避免字符串转换
        let no_parsed = self.parse_phone_prefix(no)?;

        // 使用标准库的二分查找，性能更优
        match self
            .index
            .binary_search_by_key(&no_parsed, |idx| idx.phone_no_prefix)
        {
            Ok(pos) => {
                let index_item = &self.index[pos];
                let record = self.parse_to_record(index_item.records_offset as usize)?;
                let card_type = CardType::from_u8(index_item.card_type)?;
                let result = PhoneNoInfo {
                    province: record.province,
                    city: record.city,
                    zip_code: record.zip_code,
                    area_code: record.area_code,
                    card_type: card_type.get_description(),
                };

                // 缓存结果（仅当缓存启用且限制缓存大小避免内存泄漏）  
                if self.cache_enabled {
                    if let Ok(mut cache) = self.cache.lock() {
                        if cache.len() < self.cache_max_size {
                            cache.insert(no.to_string(), result.clone());
                        }
                    }
                }

                Ok(result)
            }
            Err(_) => Err(ErrorKind::NotFound),
        }
    }

    /// 快速解析手机号前缀，避免字符串分配
    #[inline]
    fn parse_phone_prefix(&self, no: &str) -> Fallible<i32> {
        let bytes = no.as_bytes();
        if bytes.len() < 7 {
            return Err(ErrorKind::InvalidLength);
        }

        let mut result = 0i32;
        for i in 0..7 {
            let digit = bytes[i];
            if !(b'0'..=b'9').contains(&digit) {
                return Err(ErrorKind::InvalidPhoneDatabase);
            }
            result = result * 10 + (digit - b'0') as i32;
        }
        Ok(result)
    }
}

/// 运营商类型，使用更紧凑的表示
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    #[inline]
    fn from_u8(i: u8) -> Result<CardType, ErrorKind> {
        match i {
            1 => Ok(CardType::Cmcc),
            2 => Ok(CardType::Cucc),
            3 => Ok(CardType::Ctcc),
            4 => Ok(CardType::CtccV),
            5 => Ok(CardType::CuccV),
            6 => Ok(CardType::CmccV),
            7 => Ok(CardType::Cbcc),
            8 => Ok(CardType::CbccV),
            _ => Err(ErrorKind::InvalidOpNo),
        }
    }

    /// 使用静态字符串避免内存分配
    #[inline]
    const fn get_description(&self) -> &'static str {
        match self {
            CardType::Cmcc => "中国移动",
            CardType::Cucc => "中国联通",
            CardType::Ctcc => "中国电信",
            CardType::CtccV => "中国电信虚拟运营商",
            CardType::CuccV => "中国联通虚拟运营商",
            CardType::CmccV => "中国移动虚拟运营商",
            CardType::Cbcc => "中国广电",
            CardType::CbccV => "中国广电虚拟运营商",
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct PhoneNoInfo {
    /// 省
    pub province: String,
    /// 市
    pub city: String,
    /// 邮政编码
    pub zip_code: String,
    /// 长途区号
    pub area_code: String,
    /// 卡类型
    pub card_type: &'static str,
}

type Fallible<T> = Result<T, ErrorKind>;
