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

pub fn main() -> Result<(), String> {
    let response = type_state_builder::HttpResponse::new();

    // Status line is required.
    let mut response = response.set_status_line(200, "OK");
    println!("response_code: {}", response.get_response_code());

    // Body and headers are optional.
    response.add_header("Content-Type", "text/html");
    response.set_body("<html><body>Hello World!</body></html>");

    let response = response.finish();
    println!("response: {:#?}", response);

    let actual_response_state = response.get();
    println!("actual_response_state: {:#?}", actual_response_state);

    Ok(())
}

mod state {
    use super::*;

    #[derive(Debug, Default, Clone)]
    pub struct ResponseData {
        pub status_line: String,
        pub response_code: u8,
        pub body: String,
        pub headers: Vec<(String, String)>,
    }

    #[derive(Debug, Clone, Default)]
    pub struct StartState {}

    #[derive(Debug, Clone, Default)]
    pub struct HeaderAndBodyState {
        pub response_code: u8,
    }

    #[derive(Debug, Clone, Default)]
    pub struct BodyState {}

    #[derive(Debug, Clone, Default)]
    pub struct FinalState {}

    // The following marker trait is used to restrict the operations that are available in
    // each state. This isn't strictly necessary, but it's a nice thing to use in a where
    // clause to restrict types.
    pub trait Marker {}
    impl Marker for StartState {}
    impl Marker for HeaderAndBodyState {}
    impl Marker for BodyState {}
    impl Marker for FinalState {}
}

mod type_state_builder {
    use super::*;

    #[derive(Debug, Clone, Default)]
    pub struct HttpResponse<S>
    where
        S: state::Marker,
    {
        state: state::ResponseData,
        extra: S,
    }

    // Operations that are only valid in StartState.
    impl HttpResponse<state::StartState> {
        pub fn new() -> Self {
            HttpResponse {
                state: state::ResponseData::default(),
                extra: state::StartState {},
            }
        }

        pub fn set_status_line(
            self,
            code: u8,
            message: &str,
        ) -> HttpResponse<state::HeaderAndBodyState> {
            HttpResponse {
                state: state::ResponseData {
                    status_line: format!("HTTP/1.1 {} {}", code, message),
                    ..self.state
                },
                extra: state::HeaderAndBodyState {
                    response_code: code,
                },
            }
        }
    }

    // Operations that are only valid in HeaderAndBodyState.
    impl HttpResponse<state::HeaderAndBodyState> {
        pub fn add_header(&mut self, key: &str, value: &str) {
            self.state
                .headers
                .push((key.to_string(), value.to_string()));
        }

        pub fn get_response_code(&self) -> u8 {
            self.extra.response_code
        }

        pub fn set_body(&mut self, body: &str) {
            self.state.body = body.to_string();
        }

        pub fn finish(self) -> HttpResponse<state::FinalState> {
            HttpResponse {
                state: self.state,
                extra: state::FinalState {},
            }
        }
    }

    // Operations that are only valid in FinalState.
    impl HttpResponse<state::FinalState> {
        pub fn get(self) -> state::ResponseData {
            self.state
        }
    }

    // Operations that are available in all states.
    impl<S> HttpResponse<S>
    where
        S: state::Marker,
    {
        pub fn bytes(&self) -> usize {
            self.state.body.len() + self.state.status_line.len()
        }
    }
}
