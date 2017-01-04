use std::collections::HashMap;

extern crate hyper;
use hyper::Client;

extern crate rustc_serialize;
use rustc_serialize::json;

use std::fs;
use std::io::prelude::*;
use std::io::Result;
use std::path::PathBuf;

pub struct TypeNameClient {
    type_names: HashMap<u64, String>,
    persistence: Option<PathBuf>,
}

impl TypeNameClient {
    pub fn new() -> Self {
        TypeNameClient {
            type_names: HashMap::new(),
            persistence: None,
        }
    }

    pub fn with_persistence(path: PathBuf) -> Result<Self> {
        Ok(TypeNameClient {
            type_names: load_names(&path)?,
            persistence: Some(path),
        })
    }

    pub fn name(&mut self, type_id: u64) -> String {
        use std::io::prelude::*;

        if let Some(name) = self.type_names.get(&type_id) {
            return name.clone();
        }

        let client = Client::new();
        let mut response = client.get(&format!("https://crest-tq.eveonline.com/inventory/types/{}/",
                          type_id))
            .send()
            .expect("Could not read API");

        let mut response_string = String::new();
        response.read_to_string(&mut response_string).expect("Could not read response");

        let data = json::Json::from_str(&response_string).expect("Could not parse into JSON");
        let name = data.as_object().unwrap().get("name").unwrap().as_string().unwrap().to_owned();
        self.type_names.insert(type_id, name.clone());
        name
    }

    // There is a bug that prevents this from being implemented in Drop.
    pub fn persist(self) -> Result<()> {
        match self.persistence {
            Some(path) => write_names(&path, &self.type_names),
            None => Ok(()),
        }
    }
}

fn load_names(path: &PathBuf) -> Result<HashMap<u64, String>> {
    if path.exists() {
        let mut f = fs::OpenOptions::new().read(true).open(path)?;
        let mut s = String::new();
        f.read_to_string(&mut s)?;
        Ok(json::decode(&s).expect("Could not decode database file"))
    } else {
        Ok(HashMap::new())
    }
}

fn write_names(path: &PathBuf, names: &HashMap<u64, String>) -> Result<()> {
    let mut f = fs::OpenOptions::new().create(true).truncate(true).write(true).open(path)?;
    f.write_all(json::encode(&names).expect("Could not encode database file").as_ref())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_persistence() {
        // First time should get from web.
        let path = PathBuf::from("type_names_db.json");
        let mut client = TypeNameClient::with_persistence(path)
            .expect("Could not initialize client");
        let name = client.name(30);
        assert_eq!(name, "Faction");
        client.persist().expect("Could not persist client data");

        // Second time should get from file.
        let path = PathBuf::from("type_names_db.json");
        assert!(path.exists());
        let mut client = TypeNameClient::with_persistence(path)
            .expect("Could not initialize client");
        let name = client.name(30);
        assert_eq!(name, "Faction");
        client.persist().expect("Could not persist client data");
        let _ = fs::remove_file(PathBuf::from("type_names_db.json"));
    }
}
