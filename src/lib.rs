extern crate hyper;
use hyper::Client;

extern crate rustc_serialize;
use rustc_serialize::json::Json;

pub fn to_string(type_id: u64) -> String {
    use std::io::prelude::*;

    let client = Client::new();
    let mut response = client.get(&format!("https://crest-tq.eveonline.com/inventory/types/{}/",
                     type_id))
        .send()
        .expect("Could not read API");

    let mut response_string = String::new();
    response.read_to_string(&mut response_string).expect("Could not read response");

    let data = Json::from_str(&response_string).expect("Could not parse into JSON");
    data.as_object().unwrap().get("name").unwrap().as_string().unwrap().to_owned()
}
