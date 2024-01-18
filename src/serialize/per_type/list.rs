// SPDX-License-Identifier: (Apache-2.0 OR MIT)

use crate::opt::Opt;
use crate::serialize::error::SerializeError;
use crate::serialize::serializer::{PyObjectSerializer, RECURSION_LIMIT};

use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::ptr::NonNull;

pub struct ListSerializer {
    ptr: *mut pyo3_ffi::PyObject,
    opts: Opt,
    default_calls: u8,
    recursion: u8,
    default: Option<NonNull<pyo3_ffi::PyObject>>,
}

impl ListSerializer {
    pub fn new(
        ptr: *mut pyo3_ffi::PyObject,
        opts: Opt,
        default_calls: u8,
        recursion: u8,
        default: Option<NonNull<pyo3_ffi::PyObject>>,
    ) -> Self {
        ListSerializer {
            ptr: ptr,
            opts: opts,
            default_calls: default_calls,
            recursion: recursion + 1,
            default: default,
        }
    }
}

impl Serialize for ListSerializer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if unlikely!(self.recursion == RECURSION_LIMIT) {
            err!(SerializeError::RecursionLimit)
        }
        if ffi!(Py_SIZE(self.ptr)) == 0 {
            serializer.serialize_seq(Some(0)).unwrap().end()
        } else {
            let mut seq = serializer.serialize_seq(None).unwrap();
            for idx in 0..=ffi!(Py_SIZE(self.ptr)) - 1 {
                let elem =
                    unsafe { *((*(self.ptr as *mut pyo3_ffi::PyListObject)).ob_item).offset(idx) };
                let value = PyObjectSerializer::new(
                    elem,
                    self.opts,
                    self.default_calls,
                    self.recursion,
                    self.default,
                );
                seq.serialize_element(&value)?;
            }
            seq.end()
        }
    }
}
