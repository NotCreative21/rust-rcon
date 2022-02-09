// Copyright (c) 2015 [rust-rcon developers]
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::io;
use std::error::Error as StdError;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Auth,
    CommandTooLong,
    Io(io::Error)
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Auth => "authentication failed",
            Error::CommandTooLong => "command exceeds the maximum length",
            Error::Io(_) => "unable to send"
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            Error::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(fmt, "IO error: {}", err),
            _ => write!(fmt, "{}", self.to_string()),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
