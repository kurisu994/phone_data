include!("../src/lib.rs");

#[test]
fn test() {
    let phone_data = PhoneData::new().unwrap();
    let phone = std::env::args().nth(1).expect("missing phone number");
    let result = phone_data.find(&phone);

    let mut res = false;

    if let Ok(_) = result {
        res = true;
    };
    assert!(res);
}