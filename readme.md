# 手机号归属地查询
基于rust实现，使用二分查找法。

- 归属地信息库文件大小：4,098,913 字节
- 归属地信息库最后更新：2021年08月
- 手机号段记录条数：454336

## phone.dat文件格式
```
    | 4 bytes |                     <- phone.dat 版本号（如：1701即17年1月份）
    ------------
    | 4 bytes |                     <-  第一个索引的偏移
    -----------------------
    |  offset - 8            |      <-  记录区
    -----------------------
    |  index                 |      <-  索引区
    -----------------------
```
1. 头部为8个字节，版本号为4个字节，第一个索引的偏移为4个字节；
2. 记录区 中每条记录的格式为"<省份>|<城市>|<邮编>|<长途区号>\0"。 每条记录以'\0'结束；
3. 索引区 中每条记录的格式为"<手机号前七位><记录区的偏移><卡类型>"，每个索引的长度为9个字节；

## 安装使用
`cargo run xxxxx(此处为要测试的手机号)` 

例如使用`17623552166`来测试:

```shell
cargo run 17623552166

find: PhoneNoInfo { province: "重庆", city: "重庆", zip_code: "400000", area_code: "023", card_type: "中国联通" }
```

## 更新计划
1. 优化代码，去掉警告
2. 考虑改进算法，尝试更为高效的算法
3. 改造项目使用`actix-web `来搭建后端服务支持http调用