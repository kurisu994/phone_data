include!("../src/lib.rs");

#[test]
fn phone_test() {
    let phone_data = PhoneData::new().unwrap();
    let phone = "1920000000";
    let result = phone_data.find(&phone);
    let mut res = false;
    if let Ok(data) = result {
        println!("res: {:?}", data);
        res = true;
    };
    assert!(res);
}