/*
 *   Copyright (c) 2025 Nazmul Idris
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

trait LogMessage {
    fn as_str(&self) -> &str;
}

impl LogMessage for &str {
    fn as_str(&self) -> &str {
        self
    }
}

impl LogMessage for String {
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

struct Logger;

impl Logger {
    pub fn log<M>(&self, msg_ref: &M)
    where
        M: for<'any> LogMessage,
    {
        let log_entry = msg_ref.as_str();
        println!("LOG: {}", log_entry);
    }
}

fn process_message<'ret, 'msg, F, M>(message: &'msg M, processor: F) -> &'ret str
where
    M: for<'any> LogMessage + 'msg,
    'msg: 'ret, // 'm (lifetime of message) must outlive 'b (returned ref)
    F: FnOnce(&'msg M) -> &'ret str,
{
    processor(message)
}

#[test]
fn test() {
    let logger = Logger;

    let short_lived_string = String::from("Temporary message");
    let processed_ref: &str = process_message(&short_lived_string, |msg| {
        logger.log(msg);
        msg.as_str()
    });

    println!("Processed message: {}", processed_ref);

    {
        let very_short_lived = "Ephemeral";
        let processed_ephemeral: &str = process_message(&very_short_lived, |msg| {
            logger.log(msg);
            msg.as_str()
        });
        println!("Processed ephemeral: {}", processed_ephemeral);
    }
}
