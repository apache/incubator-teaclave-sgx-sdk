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


use quote::{Tokens, ToTokens};

pub enum Fragment {
    /// Tokens that can be used as an expression.
    Expr(Tokens),
    /// Tokens that can be used inside a block. The surrounding curly braces are
    /// not part of these tokens.
    Block(Tokens),
}

macro_rules! quote_expr {
    ($($tt:tt)*) => {
        $crate::fragment::Fragment::Expr(quote!($($tt)*))
    }
}

macro_rules! quote_block {
    ($($tt:tt)*) => {
        $crate::fragment::Fragment::Block(quote!($($tt)*))
    }
}

/// Interpolate a fragment in place of an expression. This involves surrounding
/// Block fragments in curly braces.
pub struct Expr(pub Fragment);
impl ToTokens for Expr {
    fn to_tokens(&self, out: &mut Tokens) {
        match self.0 {
            Fragment::Expr(ref expr) => expr.to_tokens(out),
            Fragment::Block(ref block) => {
                out.append("{");
                block.to_tokens(out);
                out.append("}");
            }
        }
    }
}

/// Interpolate a fragment as the statements of a block.
pub struct Stmts(pub Fragment);
impl ToTokens for Stmts {
    fn to_tokens(&self, out: &mut Tokens) {
        match self.0 {
            Fragment::Expr(ref expr) => expr.to_tokens(out),
            Fragment::Block(ref block) => block.to_tokens(out),
        }
    }
}

/// Interpolate a fragment as the value part of a `match` expression. This
/// involves putting a comma after expressions and curly braces around blocks.
pub struct Match(pub Fragment);
impl ToTokens for Match {
    fn to_tokens(&self, out: &mut Tokens) {
        match self.0 {
            Fragment::Expr(ref expr) => {
                expr.to_tokens(out);
                out.append(",");
            }
            Fragment::Block(ref block) => {
                out.append("{");
                block.to_tokens(out);
                out.append("}");
            }
        }
    }
}
