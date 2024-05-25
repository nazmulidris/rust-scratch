/*
 *   Copyright (c) 2023 Nazmul Idris
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
    println!("body: {}", response.get_body());
    println!("response: {:#?}", response);
    println!(
        "response size: {}",
        response.get_size().to_string().blue().bold()
    );

    Ok(())
}

pub mod state {
    #[derive(Debug, Clone, Default)]
    pub struct Start {}

    #[derive(Debug, Clone, Default)]
    pub struct HeaderAndBody {
        pub response_code: u8,
        pub status_line: String,
        pub headers: Option<Vec<(String, String)>>,
        pub body: Option<String>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct Final {
        pub response_code: u8,
        pub status_line: String,
        pub headers: Vec<(String, String)>,
        pub body: String,
    }

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
    use super::state::{Final, HeaderAndBody, Marker, Start};

    #[derive(Debug, Clone, Default)]
    pub struct HttpResponse<S: Marker> {
        pub state: S,
    }

    // Operations that are only valid in ().
    impl HttpResponse<()> {
        pub fn new() -> HttpResponse<Start> {
            HttpResponse { state: Start {} }
        }
    }

    // Operations that are only valid in StartState.
    impl HttpResponse<Start> {
        pub fn set_status_line(
            self,
            response_code: u8,
            message: &str,
        ) -> HttpResponse<HeaderAndBody> {
            HttpResponse {
                state: HeaderAndBody {
                    response_code,
                    status_line: format!("HTTP/1.1 {} {}", response_code, message),
                    ..Default::default()
                },
            }
        }
    }

    // Operations that are only valid in HeaderAndBodyState.
    impl HttpResponse<HeaderAndBody> {
        pub fn add_header(&mut self, key: &str, value: &str) {
            if self.state.headers.is_none() {
                self.state.headers.replace(Vec::new());
            }
            if let Some(v) = self.state.headers.as_mut() {
                v.push((key.to_string(), value.to_string()))
            }
        }

        pub fn get_response_code(&self) -> u8 {
            self.state.response_code
        }

        pub fn set_body(&mut self, body: &str) {
            self.state.body.replace(body.to_string());
        }

        pub fn get_body(&self) -> Option<&str> {
            self.state.body.as_deref()
        }

        pub fn finish(mut self) -> HttpResponse<Final> {
            HttpResponse {
                state: Final {
                    response_code: self.state.response_code,
                    status_line: self.state.status_line.clone(),
                    headers: self.state.headers.take().unwrap_or_default(),
                    body: self.state.body.take().unwrap_or_default(),
                },
            }
        }
    }

    // Operations that are only valid in FinalState.
    impl HttpResponse<Final> {
        pub fn get_headers(&self) -> &Vec<(String, String)> {
            &self.state.headers
        }

        pub fn get_body(&self) -> &str {
            &self.state.body
        }

        pub fn get_response_code(&self) -> u8 {
            self.state.response_code
        }

        pub fn get_status_line(&self) -> &str {
            &self.state.status_line
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
