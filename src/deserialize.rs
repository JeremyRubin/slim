// Copyright (c) 2017 Jeremy Rubin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Result, SlimError};
use std::result;
use std::io::prelude::*;
use std::io::ErrorKind;
use byteorder::BigEndian;
use byteorder::ByteOrder;
use std::vec;
use std::str;
fn fill_buf<R: Read>(s: &mut R, buf: &mut [u8]) -> Result<()> {
    match s.read_exact(buf) {
        Err(e) => {
            match e.kind() {
                ErrorKind::UnexpectedEof => return Err(SlimError::StreamClosed),
                _ => return Err(SlimError::StreamError),
            }
        }
        _ => Ok(()),
    }
}

macro_rules! deser_int {
    ($a:ty, $b:expr, $c:path)=>    {
        impl<R: Read> Deserialize<R> for $a {
            fn decode_stream(s: &mut R) -> Result<$a> {
                let mut buf: [u8; $b] = [0; $b];
                try!(fill_buf(s, &mut buf));
                Ok($c(&buf))
            }
        }
    }
}
deser_int!(u64, 8, BigEndian::read_u64);
deser_int!(u32, 4, BigEndian::read_u32);
deser_int!(u16, 2, BigEndian::read_u16);
deser_int!(i64, 8, BigEndian::read_i64);
deser_int!(i32, 4, BigEndian::read_i32);
deser_int!(i16, 2, BigEndian::read_i16);
deser_int!(f64, 8, BigEndian::read_f64);
deser_int!(f32, 4, BigEndian::read_f32);
impl<R: Read> Deserialize<R> for u8 {
    fn decode_stream(s: &mut R) -> Result<u8> {
        let mut buf: [u8; 1] = [0; 1];
        try!(fill_buf(s, &mut buf));
        Ok(buf[0])
    }
}
impl<R: Read> Deserialize<R> for i8 {
    fn decode_stream(s: &mut R) -> Result<i8> {
        let j = try!(u8::decode_stream(s));
        Ok(j as i8)
    }
}
impl<R: Read> Deserialize<R> for bool {
    fn decode_stream(s: &mut R) -> Result<bool> {
        let j = try!(u8::decode_stream(s));
        Ok(j != 0)
    }
}
pub trait Deserialize<R> {
    fn decode_stream(r: &mut R) -> Result<Self>
        where Self: Sized,
              R: Read;
}
impl<R: Read> Deserialize<R> for () {
    fn decode_stream(_: &mut R) -> Result<()> {
        Ok(())
    }
}


impl<R: Read, T: Deserialize<R>, E: Deserialize<R>> Deserialize<R> for result::Result<T, E> {
    fn decode_stream(s: &mut R) -> Result<result::Result<T, E>> {
        let typebuf = try!(u8::decode_stream(s));
        match typebuf {
            0 => {
                if let Ok(x) = T::decode_stream(s) {
                    return Ok(Ok(x));
                }
            }
            1 => {
                if let Ok(t) = E::decode_stream(s) {
                    return Ok(Err(t));
                }
            }
            _ => (),
        }
        return Err(SlimError::DeserializationError);
    }
}


use std::borrow::Cow;
impl<'a, R: Read, T: Deserialize<R> + Clone> Deserialize<R> for Cow<'a, T> {
    fn decode_stream(s: &mut R) -> Result<Cow<'a, T>> {
        Ok(Cow::Owned(try!(T::decode_stream(s))))
    }
}

impl<'a, R: Read> Deserialize<R> for Cow<'a, str> {
    fn decode_stream(s: &mut R) -> Result<Cow<'a, str>> {
        let size = try!(u64::decode_stream(s)) as usize;
        let mut buf = vec::Vec::new();
        buf.resize(size, 0);
        try!(fill_buf(s, buf.as_mut_slice()));
        match str::from_utf8(buf.as_slice()) {
            Ok(s) => Ok(Cow::Owned(s.to_string())),
            Err(_) => Err(SlimError::DeserializationError),
        }

    }
}
impl<R: Read> Deserialize<R> for String {
    fn decode_stream(s: &mut R) -> Result<String> {
        let size = try!(u64::decode_stream(s)) as usize;
        let mut buf = vec::Vec::new();
        buf.resize(size, 0);
        try!(fill_buf(s, buf.as_mut_slice()));
        match str::from_utf8(buf.as_slice()) {
            Ok(s) => Ok(s.to_string()),
            Err(_) => Err(SlimError::DeserializationError),
        }

    }
}

impl<R: Read, T> Deserialize<R> for vec::Vec<T>
    where T: Deserialize<R>
{
    fn decode_stream(s: &mut R) -> Result<vec::Vec<T>> {
        let size = try!(u64::decode_stream(s)) as usize;
        let mut buf = vec::Vec::new();
        buf.reserve(size);
        for _ in 0..size {
            buf.push(try!(T::decode_stream(s)));
        }
        Ok(buf)
    }
}

impl<R: Read, T: Deserialize<R>> Deserialize<R> for Option<T> {
    fn decode_stream(s: &mut R) -> Result<Option<T>> {
        let typebuf = try!(u8::decode_stream(s));
        match typebuf {
            0 => return Ok(None),
            1 => {
                if let Ok(t) = T::decode_stream(s) {
                    return Ok(Some(t));
                }
            }
            _ => (),
        }
        return Err(SlimError::DeserializationError);
    }
}

impl<R: Read, T: Deserialize<R>, E: Deserialize<R>> Deserialize<R> for (T, E) {
    fn decode_stream(s: &mut R) -> Result<(T, E)> {
        if let Ok(x) = T::decode_stream(s) {
            if let Ok(t) = E::decode_stream(s) {
                return Ok((x, t));
            }
        }
        Err(SlimError::DeserializationError)
    }
}

impl<R: Read> Deserialize<R> for SlimError {
    fn decode_stream(s: &mut R) -> Result<SlimError> {
        let x = try!(u8::decode_stream(s));
        match x {
            0 => Ok(SlimError::SerializationError),
            1 => Ok(SlimError::DeserializationError),
            2 => Ok(SlimError::StreamClosed),
            3 => Ok(SlimError::StreamError),
            _ => Err(SlimError::DeserializationError),
        }

    }
}
