// Copyright (c) 2017 Jeremy Rubin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate byteorder;
pub mod serialize;
pub mod deserialize;
pub mod transportable;
pub use serialize::*;
pub use deserialize::*;
pub use transportable::*;

#[derive(Debug)]
pub enum SlimError {
    SerializationError,
    DeserializationError,
    StreamError,
    StreamClosed,
}

pub type Result<T> = std::result::Result<T, SlimError>;


#[cfg(test)]
mod serialization_tests {
    use super::*;
    #[test]
    fn serdeser_u64() {
        use std::io::Cursor;
        let mut v = Vec::new();
        v.resize(8, 0);
        let mut buff: Cursor<Vec<u8>> = Cursor::new(v);
        let a: u64 = 100;
        {
            a.encode_stream(&mut buff).unwrap();
        }
        buff.set_position(0);
        let x: Result<u64> = u64::decode_stream(&mut buff);
        match x {
            Ok(x) => assert_eq!(x, a),
            _ => panic!("Failed to deserialize {} properly", a),
        }
    }
    #[test]
    fn serdeser_string() {
        use std::io::Cursor;
        let mut v = Vec::new();
        v.resize(8, 0);
        let mut buff: Cursor<Vec<u8>> = Cursor::new(v);
        let a: String = "12345678".to_string();
        {
            a.encode_stream(&mut buff).unwrap();
        }
        buff.set_position(0);
        let x: Result<String> = String::decode_stream(&mut buff);
        match x {
            Ok(x) => assert_eq!(x, a),
            _ => panic!("Failed to deserialize {} properly", a),
        }
    }
    #[test]
    fn serdeser_result_ok_string() {
        use std::io::Cursor;
        let mut v = Vec::new();
        v.resize(100, 0);
        let mut buff: Cursor<Vec<u8>> = Cursor::new(v);
        let a: String = "1234567".to_string();
        {
            let b = a.clone();
            let v_enc: Result<String> = Ok(b);
            v_enc.encode_stream(&mut buff).unwrap();
        }
        buff.set_position(0);
        let x: Result<Result<String>> = Result::<String>::decode_stream(&mut buff);
        match x {
            Ok(Ok(x)) => assert_eq!(x, a),
            _ => panic!("Failed to deserialize {} properly", a),
        }
    }
    #[test]
    fn serdeser_result_err() {
        use std::io::Cursor;
        let mut v = Vec::new();
        v.resize(100, 0);
        let mut buff: Cursor<Vec<u8>> = Cursor::new(v);
        let r: Result<String> = Err(SlimError::SerializationError);
        r.encode_stream(&mut buff).unwrap();
        buff.set_position(0);
        let x: Result<Result<String>> = Result::<String>::decode_stream(&mut buff);
        println!("{:?}", x);
        match x {
            Ok(Err(SlimError::DeserializationError)) => (),
            _ => panic!("Failed to deserialize Err(SerializationError) properly"),
        }
    }
}
