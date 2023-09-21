use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::rc::Rc;
// use std::{thread};
use actix_web::{post, web};
use actix_web::web::{Json};
use csv::{Reader, Writer};
use serde::{Deserialize};
use utoipa::ToSchema;
use uuid::Uuid;
use xlsxwriter::Workbook;
use xlsxwriter::worksheet::WorksheetCol;
use tempdir::TempDir;
use crate::api::email::EmailService;
use crate::AppState;


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
    pub fn execute(&self, mut reader: Reader<File>, file_type: &FileType, email_service: &EmailService) {
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
            let record = result.unwrap();

            let value = record.get(&self.column_name);

            if let Some(value) = value {
                let hashmap = value_to_target_data.get(value);
                if let Some(hash_map) = hashmap {
                    hash_map.borrow_mut().data.push(record);
                }
            }
        };

        if let FileType::EXCEL = file_type {
            for (_k, v) in value_to_target_data.iter() {
                let uuid = format!("{}.xlsx", Uuid::new_v4().to_string());
                let dir = TempDir::new("act").expect("cannot create tmpdir");
                let file_path = dir.path().join(uuid).into_os_string().into_string().unwrap();

                let workbook = Workbook::new(&file_path).expect("cannot create workbook");
                let mut sheet = workbook.add_worksheet(None).expect("can add sheet");

                for (i, key) in v.borrow().data.get(0).unwrap().keys().enumerate() {
                    sheet.write_string(0, i as WorksheetCol, key, None).expect("TODO: panic message");
                }
                let mut y_index = 1;

                for record in v.borrow().data.iter() {
                    for (i, key) in v.borrow().data.get(0).unwrap().keys().enumerate() {
                        sheet.write_string(y_index, i as WorksheetCol, record.get(key).unwrap(), None).expect("TODO: panic message");
                    }
                    y_index += 1;
                }
                let targets = v.borrow().targets.clone();
                workbook.close().unwrap();

                self.output_options.export(targets, &file_path, email_service);
            }
        } else if let FileType::CSV = file_type {
            for (_k, v) in value_to_target_data.iter() {
                let uuid = format!("{}.csv", Uuid::new_v4().to_string());
                let dir = TempDir::new("act").expect("cannot create tmpdir");
                let csv_file_path = dir.path().join(uuid).into_os_string().into_string().unwrap();
                let mut wtr = Writer::from_path(&csv_file_path).expect("cannot create temp file");

                for key in v.borrow().data.get(0).unwrap().keys() {
                    let _ = &wtr.write_field(key);
                }
                let _ = &wtr.write_record(None::<&[u8]>);

                for record in v.borrow().data.iter() {
                    for key in v.borrow().data.get(0).unwrap().keys() {
                        let _ = wtr.write_field(record.get(key).unwrap());
                    }
                    let _ = wtr.write_record(None::<&[u8]>);
                }
                let targets = v.borrow().targets.clone();

                self.output_options.export(targets, &csv_file_path, email_service);
            }
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
    file_type: FileType,
}

impl ParseConfig {
    pub(crate) fn parse_csv(&self, reader: Reader<File>, email_service: &EmailService) {
        // if let Some(filter_options) = &self.filter_options {
        //     filter_options.execute_on_csv_record(reader);
        // }
        if let Some(split_options) = &self.split_options {
            split_options.execute(reader, &self.file_type, email_service)
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
}

impl OutputOptions {
    pub fn export(&self, targets: Vec<String>, file_path: &String, email_service: &EmailService) {
        if let Some(smtp) = &self.smtp {
            if *smtp {
                let _a = email_service.send_file(targets, file_path);
            }
        }

        if let Some(sftp) = &self.sftp {
            if *sftp {
                println!("sftp");
            }
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExecuteOptions {
    parse_config: Option<ParseConfig>,
    csv_file: String,
}

impl ExecuteOptions {
    pub async fn execute(&self, email_service: &EmailService) {
        let csv_file = self.csv_file.clone(); // Assuming `csv_file` can be cloned.
        let parse_config = self.parse_config.clone(); // Assuming `parse_config` can be cloned.


        let reader = Reader::from_path(csv_file).expect("cannot find small.csv");
        if let Some(parse_config) = &parse_config {
            parse_config.parse_csv(reader, &email_service);
        }
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
pub async fn get_task(info: Json<ExecuteOptions>, data: web::Data<AppState>) -> String {
    let inner_info = info.into_inner();
    inner_info.execute(&data.mail_service).await;

    format!("operation is done")
}