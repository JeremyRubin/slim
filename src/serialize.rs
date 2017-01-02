// Copyright (c) 2017 Jeremy Rubin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::result;
use super::{SlimError, Result};
use std::io::prelude::*;
use std::io::ErrorKind;
use byteorder::BigEndian;
use byteorder::ByteOrder;

pub trait Serialize<W> {
    fn encode_stream(&self, stream: &mut W) -> Result<()> where W: Write;
}

impl<'a, W: Write> Serialize<W> for &'a str {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        try!((self.len() as u64).encode_stream(s));
        try!(write_buf(s, self.as_bytes()));
        Ok(())
    }
}
use std::borrow::Cow;
impl<'a, W: Write, T: Serialize<W> + Clone> Serialize<W> for Cow<'a, T> {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        T::encode_stream(self, s)
    }
}
impl<W: Write> Serialize<W> for String {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        try!((self.len() as u64).encode_stream(s));
        try!(write_buf(s, self.as_bytes()));
        Ok(())
    }
}

use std::vec::Vec;
impl<W: Write, T> Serialize<W> for Vec<T>
    where T: Serialize<W>
{
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        try!((self.len() as u64).encode_stream(s));
        for entry in self {
            try!(entry.encode_stream(s));
        }
        Ok(())
    }
}
fn write_buf<W: Write>(s: &mut W, buf: &[u8]) -> Result<()> {

    match s.write_all(buf) {
        Err(e) => {
            match e.kind() {
                ErrorKind::UnexpectedEof => return Err(SlimError::StreamClosed),
                _ => return Err(SlimError::StreamError),
            }
        }
        Ok(_) => return Ok(()),
    }
}
macro_rules! ser_int {
    ($a:ty, $b:expr, $c:path)=>    {
        impl<W: Write> Serialize<W> for $a {
            fn encode_stream(&self, s: &mut W) -> Result<()> {
                let mut buf: [u8; $b] = [0; $b];
                $c(&mut buf, *self);
                write_buf(s, &buf)
            }
        }
    }
}

ser_int!(u64, 8, BigEndian::write_u64);
ser_int!(u32, 4, BigEndian::write_u32);
ser_int!(u16, 2, BigEndian::write_u16);
ser_int!(i64, 8, BigEndian::write_i64);
ser_int!(i32, 4, BigEndian::write_i32);
ser_int!(i16, 2, BigEndian::write_i16);
ser_int!(f64, 8, BigEndian::write_f64);
ser_int!(f32, 4, BigEndian::write_f32);

impl<W: Write> Serialize<W> for u8 {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        write_buf(s, &[*self])
    }
}
impl<W: Write> Serialize<W> for i8 {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        write_buf(s, &[*self as u8])
    }
}
impl<W: Write> Serialize<W> for bool {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        write_buf(s, &[*self as u8])
    }
}
impl<W: Write> Serialize<W> for () {
    fn encode_stream(&self, _: &mut W) -> Result<()> {
        Ok(())
    }
}

impl<W: Write, T: Serialize<W>, E: Serialize<W>> Serialize<W> for result::Result<T, E> {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        match self {
            &Ok(ref a) => {
                try!(0u8.encode_stream(s));
                a.encode_stream(s)
            }
            &Err(ref a) => {
                try!(1u8.encode_stream(s));
                a.encode_stream(s)
            }
        }
    }
}
impl<W: Write, T: Serialize<W>, E: Serialize<W>> Serialize<W> for (T, E) {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        self.0.encode_stream(s).and_then(|_| self.1.encode_stream(s))
    }
}

impl<W: Write, T: Serialize<W>> Serialize<W> for Option<T> {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        match self {
            &None => 0u8.encode_stream(s),
            &Some(ref a) => {
                try!(1u8.encode_stream(s));
                a.encode_stream(s)
            }
        }
    }
}

impl<W: Write> Serialize<W> for SlimError {
    fn encode_stream(&self, s: &mut W) -> Result<()> {
        let buf: u8 = match *self {
            SlimError::DeserializationError => 0,
            SlimError::SerializationError => 1,
            SlimError::StreamClosed => 2,
            SlimError::StreamError => 3,
        };
        buf.encode_stream(s)
    }
}
