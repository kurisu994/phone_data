mod lib;

fn main() {
    let phone_data = phone_data::PhoneData::new("phone.dat").unwrap();
    let phone = std::env::args().nth(1).expect("missing phone number");
    println!("find: {:?}", phone_data.find(&phone).unwrap());
}
