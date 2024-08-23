// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright (C) 2020 Tobias Hunger <tobias.hunger@gmail.com>

// Setup warnings/errors:
#![forbid(unsafe_code)]
#![deny(bare_trait_objects, unused_doc_comments, unused_import_braces)]
// Clippy:
#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::non_ascii_literal)]

use chrono::{Local, NaiveDate};
use downloader::{Download, Downloader};

// Define a custom progress reporter:
struct SimpleReporterPrivate {
    last_update: std::time::Instant,
    max_progress: Option<u64>,
    message: String,
}
struct SimpleReporter {
    private: std::sync::Mutex<Option<SimpleReporterPrivate>>,
}

impl SimpleReporter {
    fn create() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self {
            private: std::sync::Mutex::new(None),
        })
    }
}

impl downloader::progress::Reporter for SimpleReporter {
    fn setup(&self, max_progress: Option<u64>, message: &str) {
        let private = SimpleReporterPrivate {
            last_update: std::time::Instant::now(),
            max_progress,
            message: message.to_owned(),
        };

        let mut guard = self.private.lock().unwrap();
        *guard = Some(private);
    }

    fn progress(&self, current: u64) {
        if let Some(p) = self.private.lock().unwrap().as_mut() {
            let max_bytes = match p.max_progress {
                Some(bytes) => format!("{:?}", bytes),
                None => "{unknown}".to_owned(),
            };
            if p.last_update.elapsed().as_millis() >= 1000 {
                println!(
                    "test file: {} of {} bytes. [{}]",
                    current, max_bytes, p.message
                );
                p.last_update = std::time::Instant::now();
            }
        }
    }

    fn set_message(&self, message: &str) {
        println!("test file: Message changed to: {}", message);
    }

    fn done(&self) {
        let mut guard = self.private.lock().unwrap();
        *guard = None;
        println!("test file: [DONE]");
    }
}

fn main() {
    let mut downloader = Downloader::builder()
        .download_folder(std::path::Path::new("/Users/name/Downloads/reports"))
        .parallel_requests(1)
        .build()
        .unwrap();
    let prefix = "https://what-is-your-site.com/reports";

    let today = Local::now().date_naive();
    let start_date = NaiveDate::from_ymd_opt(2024, 7, 11).unwrap();
    let range = start_date
        .iter_days()
        .take(44);
        // .enumerate();
    // 计算日期范围
    for date in range {
        if date == today {
            println!("reached today");
            break;
        }
        let date_str = date.format("%Y-%m-%d").to_string();
        let dl = downloader::Download::new(
            (prefix.to_owned() + "/DPA_Sales_Report_FS/DPA_sales_report_" + &date_str + ".xls")
                .as_str(),
        );

        let dl = dl.progress(SimpleReporter::create());
        let ts: [Download; 1] = [dl];

        let result = downloader.download(&ts).unwrap();

        for r in result {
            match r {
                Err(e) => println!("Error: {}", e.to_string()),
                Ok(s) => println!("Success: {}", &s),
            };
        }
    }
}
