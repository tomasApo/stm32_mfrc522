# stm32_mfrc522

A embedded rust project for an RFID MFRC522 & DISCOVERY Board stm32f3
Using https://docs.rs/mfrc522/0.2.0/mfrc522/ crate with exmaple code from Jonas Spanoghe

![RFID_discoveryBoard](https://user-images.githubusercontent.com/75183079/206921404-1f4b7da5-f157-4555-9cf6-f932bfa5fd5d.jpg)

To-do 
- [ ] Remove the constant error codes from timeout of not reading a card
- - [ ] Maybe? ```if let Ok(atqa) = mfrc522.wupa() { ``` instead of match on mfrc522.wupa(),


