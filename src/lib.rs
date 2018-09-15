extern crate libsqlite3_sys as ffi;

type Connection = SqliteCon;

mod rows;
mod stmt;
mod traits;

pub use rows::{Row, Rows};
use std::os::raw::c_int;
pub use stmt::Stmt;

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Uint(u64),
    Float(f64),
    String(String),
    Buf(Vec<u8>),
    Null,
}

pub enum SqliteTypes {
    Int,
    FLoat,
    Blob,
    Null,
    Text,
}
#[derive(Debug)]
pub enum Error {
    SqliteError(String),
    Unknown(String),
    BindError(String),
}

type Lesult<T> = Result<T, Error>;

impl SqliteTypes {
    pub fn new(code: c_int) -> Lesult<Self> {
        match code {
            ffi::SQLITE_INTEGER => Ok(SqliteTypes::Int),
            ffi::SQLITE_FLOAT => Ok(SqliteTypes::FLoat),
            ffi::SQLITE_BLOB => Ok(SqliteTypes::Blob),
            ffi::SQLITE_NULL => Ok(SqliteTypes::Null),
            ffi::SQLITE_TEXT => Ok(SqliteTypes::Text),
            _ => Err(Error::Unknown("Cannot SQLITE type".to_string())),
        }
    }
}

#[derive(Debug)]
pub struct SqliteCon {
    pub con: *mut ffi::sqlite3,
}

impl SqliteCon {
    pub fn new(path: &str) -> Result<Self, &str> {
        unsafe {
            ffi::sqlite3_initialize();

            let mut a = std::mem::uninitialized();
            let cstr = std::ffi::CString::new(path).unwrap();

            let con = ffi::sqlite3_open_v2(
                cstr.as_ptr(),
                &mut a,
                ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE,
                std::ptr::null(),
            );

            Ok(Self { con: a })
        }
    }

    pub fn get_last_error(&self, err_code: i32) -> String {
        let err_cstr = unsafe { ffi::sqlite3_errmsg(self.con) };

        if err_cstr.is_null() {
            String::new()
        } else {
            unsafe { std::ffi::CString::from_raw(err_cstr as *mut i8) }
                .into_string()
                .unwrap()
        }
    }
}

impl Drop for SqliteCon {
    fn drop(&mut self) {
        unsafe {
            ffi::sqlite3_close(self.con);
            ffi::sqlite3_shutdown();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    

}
