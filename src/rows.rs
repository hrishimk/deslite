use super::ffi;
use super::{Error, Lesult, SqliteTypes, Stmt, Value};
use std;

#[derive(Debug)]
pub struct Row<'con> {
    stmt: &'con Stmt<'con>,
}

impl<'con> Row<'con> {
    pub fn new(stmt: &'con Stmt<'con>) -> Self {
        Row { stmt }
    }

    pub fn get<E, T>(&self, key: T) -> Lesult<E>
    where
        T: ColIndex,
        E: std::convert::From<Value>,
    {
        let index = key.idx(self.stmt)?;
        let val = self.get_value(index)?;
        Ok(E::from(val))
    }

    pub fn get_value(&self, index: usize) -> Lesult<Value> {
        match self.stmt.colum_type(index).unwrap() {
            SqliteTypes::Int => Ok(self.stmt.get_int64(index)),
            SqliteTypes::FLoat => Ok(self.stmt.get_double(index)),
            SqliteTypes::Text => Ok(self.stmt.get_text(index)),
            SqliteTypes::Blob => Ok(self.stmt.get_blob(index)),
            SqliteTypes::Null => Ok(Value::Null),
        }
    }
}

#[derive(Debug)]
pub struct Rows<'con> {
    stmt: Stmt<'con>,
}

impl<'con> Rows<'con> {
    pub fn get_stmt(&'con self) -> &'con Stmt<'con> {
        &self.stmt
    }

    pub fn new(stmt: Stmt<'con>) -> Self {
        Rows { stmt }
    }

    pub fn iter(&'con self) -> RowIterator<'con> {
        RowIterator::new(self)
    }

    pub fn execute(&self) -> Lesult<()> {
        let res = self.stmt.step()?;
        if res == ffi::SQLITE_DONE || res == ffi::SQLITE_ROW {
            Ok(())
        } else {
            Err(Error::Unknown(
                "Failed to execute prepared stmt".to_string(),
            ))
        }
    }
}

#[derive(Debug)]
pub struct RowIterator<'con> {
    stmt: &'con Stmt<'con>,
}

impl<'con> RowIterator<'con> {
    pub fn new(rows: &'con Rows) -> Self {
        RowIterator { stmt: &rows.stmt }
    }
}

impl<'con> Iterator for RowIterator<'con> {
    // we will be counting with usize
    type Item = Row<'con>;

    // next() is the only required method
    fn next(&mut self) -> Option<Row<'con>> {
        let step = match self.stmt.step() {
            Ok(x) => x,
            Err(_x) => return None,
        };
        if step == ffi::SQLITE_ROW {
            let a = self.stmt; //self.get_stmt();
            return Some(Row::new(a));
        } else {
            return None;
        }
    }
}

pub trait ColIndex {
    fn idx(&self, stmt: &Stmt) -> Lesult<usize>;
}

impl ColIndex for usize {
    fn idx(&self, stmt: &Stmt) -> Lesult<usize> {
        if *self > stmt.colum_count() {
            return Err(Error::IndexOutOfBounds(format!("{}", self)));
        } else {
            return Ok(*self);
        }
    }
}

impl<'a> ColIndex for &'a str {
    fn idx(&self, stmt: &Stmt) -> Lesult<usize> {
        stmt.colum_index(*self)
    }
}
