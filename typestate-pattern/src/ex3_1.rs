/*
 *   Copyright (c) 2024 Nazmul Idris
 *   All rights reserved.
 *
 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

//! Similar to `ex3.rs`, but use enums instead of structs.
//!
//! - You can use enums instead of structs if you have shared data (inner) that you move
//!   with state transitions.
//! - And you have to use `PhantomData` here.

use self::type_state_builder::HttpResponse;
use crossterm::style::Stylize;

pub fn main() -> Result<(), String> {
    let response = HttpResponse::<()>::new();
    println!("{}", "Start state".red().bold().underlined());
    println!("response: {:#?}", response);
    println!(
        "response size: {}",
        response.get_size().to_string().blue().bold()
    );

    // Status line is required.
    println!("{}", "HeaderAndBody state".red().bold().underlined());
    let mut response = response.set_status_line(200, "OK");
    println!("response_code: {}", response.get_response_code());
    println!("response body: {:#?}", response.get_body());
    println!("response: {:#?}", response);
    println!(
        "response size: {}",
        response.get_size().to_string().blue().bold()
    );

    // Body and headers are optional.
    println!("{}", "HeaderAndBody state # 2".red().bold().underlined());
    response.add_header("Content-Type", "text/html");
    response.set_body("<html><body>Hello World!</body></html>");
    println!("response: {:#?}", response);
    println!(
        "response size: {}",
        response.get_size().to_string().blue().bold()
    );

    // Final state.
    println!("{}", "Final state".red().bold().underlined());
    let response = response.finish();
    println!("response_code: {}", response.get_response_code());
    println!("status_line: {}", response.get_status_line());
    println!("headers: {:#?}", response.get_headers());
    println!("body: {:#?}", response.get_body());
    println!("response: {:#?}", response);
    println!(
        "response size: {}",
        response.get_size().to_string().blue().bold()
    );

    Ok(())
}

pub mod data {
    #[derive(Debug, Clone, Default)]
    pub struct HttpResponseData {
        pub response_code: u8,
        pub status_line: String,
        pub headers: Option<Vec<(String, String)>>,
        pub body: Option<String>,
    }
}

pub mod state {
    #[derive(Debug, Clone)]
    pub enum Start {}

    #[derive(Debug, Clone)]
    pub enum HeaderAndBody {}

    #[derive(Debug, Clone)]
    pub struct Final {}

    // The following marker trait is used to restrict the operations that are available in
    // each state. This isn't strictly necessary, but it's a nice thing to use in a where
    // clause to restrict types.
    pub trait Marker {}
    impl Marker for () {}
    impl Marker for Start {}
    impl Marker for HeaderAndBody {}
    impl Marker for Final {}
}

pub mod type_state_builder {
    use super::{
        data::HttpResponseData,
        state::{Final, HeaderAndBody, Marker, Start},
    };
    use std::marker::PhantomData;

    #[derive(Debug, Clone)]
    pub struct HttpResponse<S: Marker> {
        pub data: HttpResponseData,
        pub state: PhantomData<S>,
    }

    // Operations that are only valid in ().
    impl HttpResponse<()> {
        pub fn new() -> HttpResponse<Start> {
            HttpResponse {
                data: HttpResponseData::default(),
                state: PhantomData::<Start>,
            }
        }
    }

    // Operations that are only valid in Start.
    impl HttpResponse<Start> {
        // setter.
        pub fn set_status_line(
            self,
            response_code: u8,
            message: &str,
        ) -> HttpResponse<HeaderAndBody> {
            HttpResponse {
                data: {
                    let mut data = self.data;
                    data.response_code = response_code;
                    data.status_line = format!("HTTP/1.1 {} {}", response_code, message);
                    data
                },
                state: PhantomData::<HeaderAndBody>,
            }
        }
    }

    // Operations that are only valid in HeaderAndBodyState.
    impl HttpResponse<HeaderAndBody> {
        // setter.
        pub fn add_header(&mut self, key: &str, value: &str) {
            let mut_data = &mut self.data;
            if mut_data.headers.is_none() {
                mut_data.headers.replace(Vec::new());
            }
            if let Some(headers) = mut_data.headers.as_mut() {
                headers.push((key.to_string(), value.to_string()))
            }
        }

        // getter.
        pub fn get_response_code(&self) -> u8 {
            self.data.response_code
        }

        // setter.
        pub fn set_body(&mut self, body: &str) {
            self.data.body.replace(body.to_string());
        }

        // getter.
        pub fn get_body(&self) -> Option<&str> {
            self.data.body.as_deref()
        }

        // transition to Final state.
        pub fn finish(self) -> HttpResponse<Final> {
            let mut data = self.data;
            HttpResponse {
                data: HttpResponseData {
                    response_code: data.response_code,
                    status_line: data.status_line.clone(),
                    headers: Some(data.headers.take().unwrap_or_default()),
                    body: Some(data.body.take().unwrap_or_default()),
                },
                state: PhantomData::<Final>,
            }
        }
    }

    // Operations that are only valid in FinalState.
    impl HttpResponse<Final> {
        pub fn get_headers(&self) -> &Option<Vec<(String, String)>> {
            &self.data.headers
        }

        pub fn get_body(&self) -> &Option<String> {
            &self.data.body
        }

        pub fn get_response_code(&self) -> u8 {
            self.data.response_code
        }

        pub fn get_status_line(&self) -> &str {
            &self.data.status_line
        }
    }

    // Operations that are available in all states.
    impl<S> HttpResponse<S>
    where
        S: Marker,
    {
        pub fn get_size(&self) -> String {
            let len = std::mem::size_of_val(self);
            format!("{} bytes", len)
        }
    }
}
