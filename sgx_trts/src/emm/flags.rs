// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

use bitflags::bitflags;

bitflags! {
    pub struct AllocFlags: u32 {
        const RESERVED = 0b0001;
        const COMMIT_NOW = 0b0010;
        const COMMIT_ON_DEMAND = 0b0100;
        const GROWSDOWN = 0b00010000;
        const GROWSUP = 0b00100000;
        const FIXED = 0b01000000;
        const SYSTEM = 0b10000000;
    }
}
