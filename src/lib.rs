use std::fs::File;
use std::io::{
    BufReader,
    Read,
};

use failure::{Fail, Fallible};
use serde_derive::Serialize;

#[derive(Fail, Debug)]
pub enum ErrorKind {
    #[fail(display = "invalid phone database.")]
    InvalidPhoneDatabase,
    #[fail(display = "length of phone number is invalid.")]
    InvalidLength,
    #[fail(display = "can not find this phone number in database.")]
    NotFound,
    #[fail(display = "invalid number to representative Communications Operators.")]
    InvalidOpNo,
}

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
    pub fn new() -> Fallible<PhoneData> {
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

    fn parse_to_record(&self, offset: usize) -> Fallible<Records> {
        if let Some(record) = self.records[offset - 8..].splitn(2, |i| *i == 0u8).nth(0) {
            let record = String::from_utf8(record.to_vec())?;
            let record: Vec<&str> = record.split('|').collect();
            if record.len() != 4 {
                return Err(ErrorKind::InvalidPhoneDatabase.into());
            }
            Ok(Records {
                province: record[0].to_string(),
                city: record[1].to_string(),
                zip_code: record[2].to_string(),
                area_code: record[3].to_string(),
            })
        } else {
            Err(ErrorKind::InvalidPhoneDatabase.into())
        }
    }

    /// 二分法查找 `phone_no` 数据
    pub fn find(&self, no: &str) -> Fallible<PhoneNoInfo> {
        let len = no.len();
        if len < 7 || len > 11 {
            return Err(ErrorKind::InvalidLength.into());
        }
        let no: i32 = no[..7].parse()?;

        let mut left = 0;
        let mut mid = 0;
        let mut right = self.index.len();
        loop {
            let new_mid = (left + right) / 2;
            if new_mid == mid {
                break Err(ErrorKind::NotFound.into());
            }
            mid = new_mid;
            let mid_index = &self.index[mid];
            let cur_phone = mid_index.phone_no_prefix;
            if cur_phone > no {
                right = mid;
            } else if cur_phone < no {
                left = mid;
            } else {
                let record = self.parse_to_record(mid_index.records_offset as usize)?;
                let card_type = CardType::from_u8(mid_index.card_type)?;
                break Ok(PhoneNoInfo {
                    province: record.province,
                    city: record.city,
                    zip_code: record.zip_code,
                    area_code: record.area_code,
                    card_type: card_type.get_description(),
                });
            }
        }
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
    fn from_u8(i: u8) -> Fallible<CardType> {
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