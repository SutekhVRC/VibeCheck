use std::collections::HashMap;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
pub struct LovenseConnectToy {
    pub nickName: String,
    pub hVersion: String,
    pub fVersion: i64,
    pub name: String,
    pub id: String,
    pub battery: i8,
    pub version: String,
    pub status: i64,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
struct LovenseConnectDevice {
    deviceId: String,
    domain: String,
    httpPort: u16,
    wsPort: u16,
    httpsPort: u16,
    wssPort: u16,
    toyJson: String,
    platform: String,
    appVersion: String,
    appType: String,
    deviceCode: String,
    toys: HashMap<String, LovenseConnectToy>,
}

pub fn get_toys_from_natp_api() -> Option<HashMap<String, LovenseConnectToy>> {
    let http_cli = reqwest::blocking::Client::builder()
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:104.0) Gecko/20100101 Firefox/104.0",
            )
            .build()
            .unwrap();
        match http_cli
            .get("https://api.lovense.com/api/lan/getToys")
            .send()
        {
            Ok(res) => {
                if res.status() == StatusCode::OK {
                    
                    let res_str = res.text().unwrap();
                    //println!("{}", res_str);
                    let api_response: HashMap<String, LovenseConnectDevice> = match serde_json::from_str(res_str.as_str()) {
                        Ok(deserialized_res) => deserialized_res,
                        Err(_) => return None,
                    };
                    
                    //println!("{:?}", api_response);
                    
                    for dev in api_response {
                        return Some(dev.1.toys);
                    }
                    return None;
                
                } else {
                    //println!("{:?}", res.text());
                    return None;
                }
            }
            Err(_err) => {return None},
        }
}