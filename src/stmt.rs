use super::{ffi, Connection, Error, Lesult, SqliteTypes, Value};
use std;

use std::ffi::CStr;
use std::os::raw::c_int;

use Row;
use Rows;

#[derive(Debug)]
pub struct Stmt<'a> {
    con: &'a Connection,
    pub stmt: *mut ffi::sqlite3_stmt,
}

impl<'a> Drop for Stmt<'a> {
    fn drop(&mut self) {
        let c = unsafe { ffi::sqlite3_finalize(self.stmt) };
    }
}

impl<'a> Stmt<'a> {
    pub fn init(con: &'a Connection) -> Self {
        Self {
            con,
            stmt: unsafe { std::mem::uninitialized() },
        }
    }

    pub fn prepare(&mut self, sql: &str) -> Result<&mut Self, &str> {
        let cstr = std::ffi::CString::new(sql).unwrap();

        let rc = unsafe {
            ffi::sqlite3_prepare_v2(
                self.con.con,
                cstr.as_ptr(),
                -1,
                &mut self.stmt,
                std::ptr::null_mut(),
            )
        };

        if rc != ffi::SQLITE_OK {
            let err = unsafe { ffi::sqlite3_errmsg(self.con.con) };
            let err = unsafe { std::ffi::CStr::from_ptr(err) };
            println!("Prepare failure error: {:?}", err);
            return Err("Failed to prepare");
        }

        Ok(self)
    }

    fn bind_buff<T>(&mut self, v: Vec<T>, i: std::os::raw::c_int) -> c_int
    where
        std::vec::Vec<u8>: std::convert::From<std::vec::Vec<T>>,
    {
        let buf_len = v.len();

        let c_string = std::ffi::CString::new(v).unwrap();

        let destructor = if buf_len > 0 {
            ffi::SQLITE_TRANSIENT()
        } else {
            ffi::SQLITE_STATIC()
        };
        unsafe {
            ffi::sqlite3_bind_text(
                self.stmt,
                i as i32,
                c_string.as_ptr(),
                buf_len as i32,
                destructor,
            )
        }
    }

    pub fn bind_values<T>(&mut self, params: Vec<T>)
    where
        T: std::clone::Clone,
        Value: std::convert::From<T>,
    {
        for (i, n) in params.iter().cloned().enumerate() {
            self.bind(n, (i + 1) as i32).unwrap();
        }
    }

    pub fn bind<T>(&mut self, param: T, index: i32) -> Lesult<()>
    where
        Value: std::convert::From<T>,
    {
        let param = Value::from(param);
        use Value::*;

        println!("index is {:#?}", index);

        let res = match param {
            Int(v) => unsafe { ffi::sqlite3_bind_int64(self.stmt, index, v) },
            Uint(v) => unsafe { ffi::sqlite3_bind_int64(self.stmt, index, v as i64) },
            Float(v) => unsafe { ffi::sqlite3_bind_double(self.stmt, index, v) },
            Buf(v) => self.bind_buff(v, index),
            String(v) => self.bind_buff(v.into_bytes(), index),
            Null => unsafe { ffi::sqlite3_bind_null(self.stmt, index) },
        };

        if res == ffi::SQLITE_OK {
            Ok(())
        } else {
            Err(Error::BindError(self.con.get_last_error(res)))
        }
    }

    pub fn step(&self) -> c_int {
        unsafe { ffi::sqlite3_step(self.stmt) }
    }

    pub fn get_rows(&self) -> Rows {
        Rows::new(self)
    }

    pub fn reset(&mut self) {
        unsafe {
            ffi::sqlite3_reset(self.stmt);
            ffi::sqlite3_clear_bindings(self.stmt);
        }
    }

    pub fn colum_count(&self) -> usize {
        unsafe { ffi::sqlite3_column_count(self.stmt) as usize }
    }

    pub fn colum_name(&self, index: usize) -> &str {
        unsafe { CStr::from_ptr(ffi::sqlite3_column_name(self.stmt, index as c_int)) }
            .to_str()
            .unwrap()
    }

    pub fn colum_index(&self, key: &str) -> Result<usize, String> {
        let count = self.colum_count();

        for i in 0..count {
            if key == self.colum_name(i) {
                return Ok(i);
            }
        }
        Err(format!("Cannot find colum {}", key))
    }

    pub fn colum_type(&self, index: usize) -> Lesult<SqliteTypes> {
        SqliteTypes::new(unsafe { ffi::sqlite3_column_type(self.stmt, index as i32) })
    }

    pub fn get_double(&self, index: usize) -> Value {
        Value::Float(unsafe { ffi::sqlite3_column_double(self.stmt, index as i32) })
    }

    pub fn get_int32(&self, index: usize) -> Value {
        Value::Int(unsafe { ffi::sqlite3_column_int(self.stmt, index as i32) } as i64)
    }

    pub fn get_int64(&self, index: usize) -> Value {
        Value::Int(unsafe { ffi::sqlite3_column_int64(self.stmt, index as i32) } as i64)
    }

    pub fn get_blob(&self, index: usize) -> Value {
        let c_ptr = unsafe { ffi::sqlite3_column_blob(self.stmt, index as i32) };

        let c_len = unsafe { ffi::sqlite3_column_bytes(self.stmt, index as i32) };
        if c_len > 0 {
            let nvec =
                unsafe { Vec::from_raw_parts(c_ptr as *mut u8, c_len as usize, c_len as usize) };
            Value::Buf(nvec)
        } else {
            Value::Buf(vec![])
        }
    }

    pub fn get_text(&self, index: usize) -> Value {
        let cstring = unsafe { ffi::sqlite3_column_text(self.stmt, index as i32) };

        let cstring = unsafe { std::ffi::CString::from_raw(cstring as *mut i8) };

        Value::String(cstring.into_string().unwrap())
    }
}
