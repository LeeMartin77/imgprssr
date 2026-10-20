use std::collections::HashMap;

use config::Config;

use crate::appconfig;

pub fn generate_app_config() -> Result<(([u8; 4], u16), appconfig::ImgprssrConfig), appconfig::ImgprssrConfigErr> {
    let raw_hashmap = Config::builder()
        // ENV Variables are IMGPRSSR_SOMETHING == something
        .add_source(config::Environment::with_prefix("IMGPRSSR"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let mut address = [0, 0, 0, 0];
    let mut port = 3000;

    if let Some(config_address) = raw_hashmap.get("address") {
        let split = config_address.split(".");
        let mut parsed = split.map(|a| a.parse::<u8>()).into_iter();
        if parsed.clone().count() != 4 || !parsed.clone().all(|res| res.is_ok()){
            return Err(appconfig::ImgprssrConfigErr::InvalidValues(vec!["address".to_owned()]))
        }
        let mut i = 0;
        while let Some(part) = parsed.next() {
            address[i] = part.unwrap();
            i += 1;
        }
    }

    if let Some(config_port) = raw_hashmap.get("port") {
        if let Ok(prsd_port) = config_port.parse::<u16>() {
            port = prsd_port;
        } else {
            return Err(appconfig::ImgprssrConfigErr::InvalidValues(vec!["port".to_owned()]))
        }
    }
    
    let app_config_res = appconfig::from_hashmap(raw_hashmap);

    match app_config_res {
        Ok(app_config) => Ok(((address, port), app_config)),
        Err(err) => Err(err),
    }
}
