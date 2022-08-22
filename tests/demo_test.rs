include!("../src/lib.rs");

#[test]
fn test() {
    let phone_data = PhoneData::new("phone.dat").unwrap();
    let phone = std::env::args().nth(1).expect("missing phone number");
    let phone_no_info = phone_data.find(&phone).unwrap();
    println!("find: {:?}", phone_no_info);
}