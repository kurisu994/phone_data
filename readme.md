# 手机号归属地查询
基于rust实现，使用二分查找法。

- 归属地信息库文件大小：4557kb
- 归属地信息库最后更新：2025年02月
- 手机号段记录条数：517258

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
`cargo run` 

例如使用`18086834111`来测试:

```shell
# example1
curl --location --request GET 'http://127.0.0.1:8080/query2/18086834111'

-----------------------------output------------------------------------------
{
    "code": 1,
    "data": {
        "province": "四川",
        "city": "成都",
        "zip_code": "610000",
        "area_code": "028",
        "card_type": "中国电信"
    },
    "success": true,
    "result": "ok"
}
```
```shell

# example2
curl 'http://127.0.0.1:9001/query?phone=18086834111'

-----------------------------output------------------------------------------
{
    "code": 1,
    "data": {
        "province": "四川",
        "city": "成都",
        "zip_code": "610000",
        "area_code": "028",
        "card_type": "中国电信"
    },
    "success": true,
    "result": "ok"
}
```

## 测试

```shell
 cargo test --package phone_data --test demo_test phone_test -- --nocapture 
```

## 更新计划
1. 添加请求拦截器
2. 添加鉴权
3. 添加异常错误统一处理
