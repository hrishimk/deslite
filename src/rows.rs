use super::ffi;
use super::{Lesult, SqliteTypes, Stmt, Value};
use std;

#[derive(Debug)]
pub struct Row<'b> {
    stmt: &'b Stmt<'b>,
}

impl<'b> Row<'b> {
    pub fn new(stmt: &'b Stmt<'b>) -> Self {
        Row { stmt }
    }

    pub fn get<E, T>(&self, key: T) -> E
    where
        T: ColIndex,
        E: std::convert::From<Value>,
    {
        let index = key.idx(self.stmt).unwrap();
        let val = self.get_value(index).unwrap();
        E::from(val)
    }

    pub fn get_value(&self, index: usize) -> Lesult<Value> {
        use SqliteTypes::*;
        match self.stmt.colum_type(index).unwrap() {
            SqliteTypes::Int => Ok(self.stmt.get_int64(index)),
            SqliteTypes::FLoat => Ok(self.stmt.get_double(index)),
            SqliteTypes::Text => Ok(self.stmt.get_text(index)),
            SqliteTypes::Blob => Ok(self.stmt.get_blob(index)),
            SqliteTypes::Null => Ok(Value::Null),
        }
    }
}

pub struct Rows<'a> {
    stmt: &'a Stmt<'a>,
}

impl<'a> Rows<'a> {
    pub fn new(stmt: &'a Stmt<'a>) -> Self {
        Rows { stmt }
    }
}

impl<'a> Iterator for Rows<'a> {
    // we will be counting with usize
    type Item = Row<'a>;

    // next() is the only required method
    fn next(&mut self) -> Option<Row<'a>> {
        let a = self.stmt.step();

        if a == ffi::SQLITE_ROW {
            return Some(Row::new(self.stmt));
        } else {
            return None;
        }
    }
}

pub trait ColIndex {
    fn idx(&self, stmt: &Stmt) -> Result<usize, String>;
}

impl ColIndex for usize {
    fn idx(&self, stmt: &Stmt) -> Result<usize, String> {
        if *self > stmt.colum_count() {
            return Err("INdex out of bounds".to_string());
        } else {
            return Ok(*self);
        }
    }
}

impl<'a> ColIndex for &'a str {
    fn idx(&self, stmt: &Stmt) -> Result<usize, String> {
        stmt.colum_index(*self)
    }
}
