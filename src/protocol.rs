// Copyright [2022] Andreas Monitzer

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub start: Date,
    pub end: Date,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Row {
    pub date: Date,
    pub code: String,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response(pub Vec<Row>);

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Ord)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl ToString for Date {
    fn to_string(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.year > other.year {
            return Some(Ordering::Greater);
        } else if self.year < other.year {
            return Some(Ordering::Less);
        }
        if self.month > other.month {
            return Some(Ordering::Greater);
        } else if self.month < other.month {
            return Some(Ordering::Less);
        }
        if self.day > other.day {
            return Some(Ordering::Greater);
        } else if self.day < other.day {
            return Some(Ordering::Less);
        }
        Some(Ordering::Equal)
    }
}

impl Date {
    /// Super-simple increment for days, does not take leap years into account!
    /// Behavior is undefined if the given date is not valid.
    pub fn next(mut self) -> Self {
        const DAYS_IN_MONTH: [u8; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        if self.day < DAYS_IN_MONTH[self.month as usize - 1] {
            self.day += 1;
            return self;
        }
        if self.month < 12 {
            self.month += 1;
            self.day = 1;
            return self;
        }
        self.year += 1;
        self.month = 1;
        self.day = 1;
        self
    }
}
