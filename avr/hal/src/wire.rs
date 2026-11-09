//! Wire
//!
//! Made up of a listening pin and a writing pin
#![allow(private_bounds)]

use crate::pins::{Pin2, Pin3};

pub(crate) trait WireOps {
    type Pin;
}

pub struct A;
impl WireOps for A {
    type Pin = Pin2;
}

pub struct B;
impl WireOps for B {
    type Pin = Pin3;
}

pub struct Wire<O: WireOps> {
    pin: O::Pin,
}

impl<O: WireOps> Wire<O> {
    pub(crate) fn new(pin: O::Pin) -> Self {
        Self { pin }
    }
}
