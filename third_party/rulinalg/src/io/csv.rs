//! CSV read / write module

pub use libcsv::{Reader, Writer, Error};

use rustc_serialize::{Decodable, Encodable};
use std::io::{Read, Write};

use super::super::matrix::{Matrix, BaseMatrix};


impl<T> Matrix<T> where T: Decodable {

    /// Read csv file as Matrix.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rulinalg::io::csv::Reader;
    /// use rulinalg::matrix::Matrix;
    ///
    /// let rdr = Reader::from_file("./data.csv").unwrap().has_headers(false);
    /// let res = Matrix::<f64>::read_csv(rdr).unwrap();
    /// ```
    pub fn read_csv<'a, R: Read>(mut reader: Reader<R>)
        -> Result<Matrix<T>, Error> {

        // headers read 1st row regardless of has_headers property
        let header: Vec<String> = try!(reader.headers());

        let mut nrows = 0;
        let ncols = header.len();

        let mut records: Vec<T> = vec![];
        for record in reader.decode() {
            let values: Vec<T> = try!(record);
            records.extend(values);
            nrows += 1;
        }
        Ok(Matrix::new(nrows, ncols, records))
    }
}

impl<T> Matrix<T> where T: Encodable {

    /// Write Matrix as csv file.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use rulinalg::io::csv::Writer;
    ///
    /// let mat = matrix![1., 7., 1.1;
    ///                   1., 3., 2.2;
    ///                   1., 1., 4.5];
    /// let mut wtr = Writer::from_file("./data.csv").unwrap();
    /// mat.write_csv(&mut wtr).unwrap();
    /// ```
    pub fn write_csv<W: Write>(&self, writer: &mut Writer<W>)
        -> Result<(), Error> {

        for row in self.row_iter() {
            try!(writer.encode(row.raw_slice()));
        }
        Ok(())
    }
}
