// Copyright (c) 2017 Jeremy Rubin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::serialize::Serialize;
use super::deserialize::Deserialize;
use std::io::prelude::*;
use std::result;
use super::{SlimError, Result};
pub trait Transportable<S>: Serialize<S> + Deserialize<S> {}
macro_rules! d {
    ($t:ty) => {
        impl<S: Read + Write> Transportable<S> for $t {}
    }
}
d!(());
d!(u64);
d!(u32);
d!(u16);
d!(u8);
d!(i64);
d!(i32);
d!(i16);
d!(i8);
d!(bool);
d!(String);
d!(SlimError);
impl<S: Read + Write, T: Transportable<S>, E: Transportable<S>> Transportable<S> for result::Result<T, E> {}
impl<S: Read + Write, T: Transportable<S>> Transportable<S> for Option<T> {}
use std::vec;
impl<S: Read + Write, T: Transportable<S>> Transportable<S> for vec::Vec<T> {}
use std::borrow::Cow;
impl<'a, S: Read + Write, T: Transportable<S> + Clone> Transportable<S> for Cow<'a, T> {}
impl<S: Read + Write, A: Transportable<S>, B: Transportable<S>> Transportable<S> for (A, B) {}
