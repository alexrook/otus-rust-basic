use core::{core::Account, protocol::*, ser_de::*};

fn main() {
    println!("Server works");
    let initial = Protocol::Response(Ok(vec![Account {
        account_id: "acc1".to_string(),
        balance: 42,
    }]));
    let mut writer: Vec<u8> = Vec::new();
    let actual: Result<(), String> = IO::write(&mut writer, &initial);
    assert!(actual.is_ok());

    let deserialized: DesResult<Protocol, String> = Protocol::deserialize(&writer);
    assert!(deserialized.is_ok());
    assert_eq!(initial, deserialized.unwrap().0);
}
