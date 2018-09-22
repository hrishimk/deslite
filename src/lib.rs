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
    Bytes(Vec<u8>),
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
    PrepareErr(String),
    IndexOutOfBounds(String),
    ConnectionErr(String),
    Empty,
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
pub struct QueryResult<'a> {
    con: &'a SqliteCon,
    affected_rows: Option<usize>,
    last_insert_id: Option<u64>,
    rows: Rows<'a>,
}

#[derive(Debug)]
pub struct SqliteCon {
    pub con: *mut ffi::sqlite3,
}

impl SqliteCon {
    pub fn new(path: &str) -> Lesult<Self> {
        let mut a = unsafe { std::mem::uninitialized() };
        let cstr = std::ffi::CString::new(path)
            .map_err(|_| Error::ConnectionErr("Failed to parse path".to_string()))?;
        let res = unsafe {
            ffi::sqlite3_open_v2(
                cstr.as_ptr(),
                &mut a,
                ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE,
                std::ptr::null(),
            )
        };

        if res != ffi::SQLITE_OK {
            return Err(Error::ConnectionErr("Failed to open database".to_string()));
        }

        Ok(Self { con: a })
    }

    pub fn get_last_error(&self, _err_code: i32) -> String {
        let err_cstr = unsafe { ffi::sqlite3_errmsg(self.con) };

        if err_cstr.is_null() {
            String::new()
        } else {
            let cstring = unsafe { std::ffi::CString::from_raw(err_cstr as *mut i8) };
            let clone = cstring.clone().into_string().unwrap();
            std::mem::forget(cstring);
            clone
        }
    }

    pub fn affected_rows(&self) -> usize {
        let ar = unsafe { ffi::sqlite3_changes(self.con) } as usize;
        ar
    }

    pub fn last_insert_id(&self) -> u64 {
        let lid = unsafe { ffi::sqlite3_last_insert_rowid(self.con) } as u64;
        lid
    }

    pub fn is_null(&self) -> bool {
        self.con.is_null()
    }
}

impl Drop for SqliteCon {
    fn drop(&mut self) {
        unsafe {
            ffi::sqlite3_close(self.con);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn data_test() {
        let con = SqliteCon::new("test_db").expect("Failed to create Sqlite db");
        let sql = "DROP TABLE IF EXISTS user";
        let mut stmt = Stmt::init(&con);
        stmt.prepare(sql).unwrap();
        stmt.execute().unwrap();
        let sql = "CREATE TABLE user (id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL DEFAULT '')";
        let mut stmt = Stmt::init(&con);
        stmt.prepare(sql).unwrap();
        stmt.execute().unwrap();
        let sql = "INSERT INTO user (name) VALUES (?), (?), (?)";
        let mut stmt = Stmt::init(&con);
        stmt.prepare(sql).unwrap();
        let params = vec!["name1", "name2", "name3"];
        stmt.bind_values(&params).unwrap();
        stmt.execute().unwrap();
        let sql = "SELECT * FROM user";
        stmt.prepare(sql).unwrap();
        //stmt.bind_values(vec![]).unwrap();
        let rows = stmt.get_rows();
        let rows_iter = rows.iter();
        let row_map = rows_iter.map(|row| row.get::<String, &str>("name").unwrap());
        let names: Vec<String> = row_map.collect();
        println!("names is {:?}", names);
        assert_eq!(
            names,
            vec![
                "name1".to_string(),
                "name2".to_string(),
                "name3".to_string(),
            ]
        );
    }
}
