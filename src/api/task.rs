use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::rc::Rc;
use std::thread;
// use super::email::send_email;
// use std::env;
// use std::fmt::format;
use actix_web::{post};
use actix_web::web::{Json};
use csv::{Reader, Writer};
use serde::{Deserialize};
// use serde_json::Value::String;
//
// extern crate reqwest;

use utoipa::ToSchema;
use crate::api::email::send_email;


#[derive(Debug, Deserialize, ToSchema, Clone)]
pub struct TargetValues {
    targets: Vec<String>,
    values: Vec<String>,
}

#[derive(Debug, Deserialize, ToSchema, Clone)]
pub struct SplitOptions {
    column_name: String,
    target_to_value: Vec<TargetValues>,
    output_options: OutputOptions,
}

impl SplitOptions {
    pub fn execute(&self, mut reader: Reader<File>) {
        let value_to_target_data = self.target_to_value.iter().fold(HashMap::<String, Rc<RefCell<TargetData>>>::new(), |mut acc, e| {
            let target_data = Rc::new(RefCell::new(TargetData {
                targets: e.targets.clone(),
                data: vec![],
            }));
            for v in e.values.iter() {
                acc.insert(v.clone(), target_data.clone());
            }
            acc
        });

        for result in reader.deserialize::<HashMap<String, String>>() {
            let mut record = result.unwrap();

            let value = record.get(&self.column_name);

            if let Some(value) = value {
                let hashmap = value_to_target_data.get(value);
                if let Some(hash_map) = hashmap {
                    hash_map.borrow_mut().data.push(record);
                }
            }
        };

        for (_k, v) in value_to_target_data.iter() {
            let mut wtr = Writer::from_writer(vec![]);

            for key in v.borrow().data.get(0).unwrap().keys() {
                let _ = wtr.write_field(key);
            }
            let _ = wtr.write_record(None::<&[u8]>);
            for record in v.borrow().data.iter() {
                for key in v.borrow().data.get(0).unwrap().keys() {
                    let _ = wtr.write_field(record.get(key).unwrap());
                }
                let _ = wtr.write_record(None::<&[u8]>);
            }

            let targets = v.borrow().targets.clone();
            let csv_data = wtr.into_inner().unwrap();

            self.output_options.export(targets, csv_data);
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, Clone)]
pub struct FilterOptions {
    filter_column_name: Vec<String>,
}

impl FilterOptions {
    pub fn execute_on_csv_record(&self, mut reader: Reader<File>) -> Vec<HashMap<String, String>> {
        let filtered_csv: Vec<HashMap<String, String>> = reader.deserialize::<HashMap<String, String>>().map(|record| {
            if let Ok(mut record) = record {
                for filter_column in self.filter_column_name.iter() {
                    record.remove(filter_column);
                }
                record
            } else {
                HashMap::new()
            }
        }).collect();
        filtered_csv
    }
}

#[derive(Debug, Deserialize, ToSchema, Clone)]
pub struct ParseConfig {
    filter_options: Option<FilterOptions>,
    split_options: Option<SplitOptions>,
}

impl ParseConfig {
    pub(crate) fn parse_csv(&self, reader: Reader<File>) {
        // if let Some(filter_options) = &self.filter_options {
        //     filter_options.execute_on_csv_record(reader);
        // }
        if let Some(split_options) = &self.split_options {
            split_options.execute(reader);
        }
    }
}

#[derive(Debug, Deserialize, ToSchema, Clone)]
pub enum FileType {
    CSV,
    EXCEL,
}

#[derive(Debug, Deserialize, ToSchema, Clone)]
pub struct OutputOptions {
    smtp: Option<bool>,
    sftp: Option<bool>,
    file_type: Option<FileType>,
}

impl OutputOptions {
    pub fn export(&self, targets: Vec<String>, mut data: Vec<u8>) {
        if let Some(file_type) = &self.file_type {
            data = match file_type {
                FileType::EXCEL => {
                    self.convert_csv_excel(data)
                }
                _ => data
            }
        }

        if let Some(smtp) = &self.smtp {
            if *smtp {
                send_email(targets, data);
            }
        }

        if let Some(sftp) = &self.sftp {
            if *sftp {
                println!("sftp");
            }
        }
    }

    fn convert_csv_excel(&self, csv_data: Vec<u8>) -> Vec<u8> {
        csv_data
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExecuteOptions {
    parse_config: Option<ParseConfig>,
    csv_file: String,
}

impl ExecuteOptions {
    pub async fn execute(&self) {
        let csv_file = self.csv_file.clone(); // Assuming `csv_file` can be cloned.
        let parse_config = self.parse_config.clone(); // Assuming `parse_config` can be cloned.

        let handle = thread::spawn(move || {
            let reader = Reader::from_path(csv_file).expect("cannot find small.csv");
            if let Some(parse_config) = &parse_config {
                parse_config.parse_csv(reader);
            }
        });
        match handle.join() {
            Ok(_) => {
                println!("ok");
            }
            Err(_) => {
                println!("err");
            }
        };
    }
}

#[derive(Clone, Debug)]
struct TargetData {
    targets: Vec<String>,
    data: Vec<HashMap<String, String>>,
}

#[utoipa::path(
request_body = ExecuteOptions,
responses(
(status = 200, description = "Search Todos did not result error", body = String),
)
)]
#[post("/execute")]
pub async fn get_task(info: Json<ExecuteOptions>) -> String {
    let inner_info = info.into_inner();
    inner_info.execute().await;

    format!("operation is done")
}