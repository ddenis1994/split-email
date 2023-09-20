use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
// use super::email::send_email;
// use std::env;
// use std::fmt::format;
use actix_web::{ post};
use actix_web::web::{Json };
use csv::Writer;
// use log::Record;
use serde::{Deserialize, Serialize};
// use serde_json::Value::String;
//
// extern crate reqwest;

use utoipa::ToSchema;
use crate::api::email::send_email;


#[derive(Debug, Deserialize,ToSchema)]
pub struct TargetValues {
    targets: Vec<String>,
    values: Vec<String>,
}

#[derive(Debug, Deserialize,ToSchema)]
pub struct SplitOptions {
    column_name: String,
    target_to_value: Vec<TargetValues>,
}

#[derive(Debug, Deserialize,ToSchema)]
pub struct FilterOptions {
    filter_column_name: Vec<String>,
}

#[derive(Debug, Deserialize,ToSchema)]
pub enum FileType {
    CSV,
    EXCEL,
}

#[derive(Debug, Deserialize,ToSchema)]
pub struct OutputOptions {
    smtp: Option<Vec<String>>,
    sftp: Option<String>,
    file_type: Option<FileType>,
}

#[derive( Debug, Deserialize, ToSchema)]
pub struct ExecuteOptions {
    #[schema()]
    split_options: Option<SplitOptions>,
    #[schema()]
    filter_options: Option<FilterOptions>,
    #[schema()]
    csv_file: Option<String>,
    #[schema()]
    output_options: OutputOptions,
}


#[derive(Clone, Debug)]
struct TargetData {
    targets: Vec<String>,
    data: Vec<HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use crate::api::task::{FilterOptions, split_filter_csv, SplitOptions, TargetValues};

    #[test]
    fn exploration() {
        let split_options = SplitOptions {
            column_name: "gendercode".to_string(),
            target_to_value: vec![
                TargetValues {
                    targets: vec!["email.sample@com".to_string()],
                    values: vec!["M".to_string()],
                },
                TargetValues {
                    targets: vec!["email2.sample@com".to_string()],
                    values: vec!["F".to_string()],
                },
            ],
        };

        let filter_options = FilterOptions {
            filter_column_name: vec!["insert date".to_string(), "cohort reference event-startdate".to_string()],
        };

        let execution_options = crate::api::task::ExecuteOptions {
            split_options: Some(split_options),
            filter_options: Some(filter_options),
            csv_file: None,
            output_options: crate::api::task::OutputOptions {
                smtp: None,
                sftp: None,
                file_type: None,
            },
        };
        let t = split_filter_csv(execution_options);
        // assert_eq!(t, "avc");
    }
}

fn split_filter_csv(execution_options: ExecuteOptions) -> HashMap<String, Rc<RefCell<TargetData>>> {
    let filter_options = execution_options.filter_options;
    let filter_column_name = match filter_options {
        Some(x) => x.filter_column_name,
        None => vec![],
    };

    let split_options = execution_options.split_options.unwrap();

    let value_to_target_data = split_options.target_to_value.iter().fold(HashMap::<String, Rc<RefCell<TargetData>>>::new(), |mut acc, e| {
        let target_data = Rc::new(RefCell::new(TargetData {
            targets: e.targets.clone(),
            data: vec![],
        }));
        for v in e.values.iter() {
            acc.insert(v.clone(), target_data.clone());
        }
        acc
    });

    let column_name = split_options.column_name;
    let mut rdr = csv::Reader::from_path("small.csv").expect("cannot find small.csv");

    for result in rdr.deserialize::<HashMap<String, String>>() {
        let mut record = result.unwrap();

        for filter_column in filter_column_name.iter() {
            record.remove(filter_column);
        }

        let r = record.get(&*column_name);
        match r {
            Some(value) => {
                let hashmap=value_to_target_data.get(value);
                match hashmap {
                    Some(x) => {
                        x.borrow_mut().data.push(record);
                    },
                    None => continue,
                }
            },
            None => continue,
        }

    };


    let smtp = execution_options.output_options.smtp;
    match smtp {
        Some(_x) => {
            for (_k, v) in value_to_target_data.iter() {
                let mut wtr = Writer::from_writer(vec![]);

                for key in v.borrow().data.get(0).unwrap().keys(){
                    wtr.write_field(key);
                }
                wtr.write_record(None::<&[u8]>);
                for record in v.borrow().data.iter(){
                    for key in v.borrow().data.get(0).unwrap().keys(){
                        wtr.write_field(record.get(key).unwrap());

                    }
                    wtr.write_record(None::<&[u8]>);
                }

                let targets=v.borrow().targets.clone();
                let csv_data =wtr.into_inner().unwrap();
                send_email(targets, csv_data);
            }

        },
        None => println!("test - {:?}", "no email"),
    }

    let smtp = execution_options.output_options.sftp.clone();
    match smtp {
        Some(x) => println!("test - {:?}", x),
        None => println!("test - {:?}", "no smtp"),
    }


    value_to_target_data
}

#[utoipa::path(
request_body=ExecuteOptions,
responses(
(status = 200, description = "Search Todos did not result error", body = String),
)
)]
#[post("/execute")]
pub async fn get_task(info: Json<ExecuteOptions>) -> String {
    let inner_info = info.into_inner();
     split_filter_csv(inner_info);



    format!("avc")
}