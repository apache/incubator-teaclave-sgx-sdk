use rulinalg::matrix::Matrix;
use rulinalg::io::csv::{Reader, Writer};

#[test]
fn test_read_csv_with_header() {
    let data = "A,B,C
1,7,1.1
1,3,2.2
1,1,4.5";
    let rdr = Reader::from_string(data).has_headers(true);
    let res = Matrix::<f64>::read_csv(rdr).unwrap();

    let exp = matrix![1., 7., 1.1;
                      1., 3., 2.2;
                      1., 1., 4.5];
    assert_matrix_eq!(res, exp);
}

#[test]
fn test_read_csv_without_header() {
    let data = "1,7,1.1
1,3,2.2
1,1,4.5";
    let rdr = Reader::from_string(data).has_headers(false);
    let res = Matrix::<f64>::read_csv(rdr).unwrap();

    let exp = matrix![1., 7., 1.1;
                      1., 3., 2.2;
                      1., 1., 4.5];
    assert_matrix_eq!(res, exp);
}

#[test]
fn test_read_csv_integer_like() {
    let data = "1,7,1
1,3,2
1,1,4";
    let rdr = Reader::from_string(data).has_headers(false);
    let res = Matrix::<f64>::read_csv(rdr).unwrap();

    let exp = matrix![1., 7., 1.;
                      1., 3., 2.;
                      1., 1., 4.];
    assert_matrix_eq!(res, exp);
}

#[test]
fn test_read_csv_with_header_int() {
    let data = "A,B,C
1,2,3
4,5,6
7,8,9";
    let rdr = Reader::from_string(data).has_headers(true);
    let res = Matrix::<usize>::read_csv(rdr).unwrap();

    let exp = matrix![1, 2, 3;
                      4, 5, 6;
                      7, 8, 9];
    assert_matrix_eq!(res, exp);
}

#[test]
fn test_read_csv_empty() {
    let data = "";
    let rdr = Reader::from_string(data).has_headers(true);
    let res = Matrix::<f64>::read_csv(rdr).unwrap();
    let exp: Matrix<f64> = Matrix::new(0, 0, vec![]);
    assert_matrix_eq!(res, exp);
}

#[test]
fn test_read_csv_error_different_items() {
    let data = "A,B,C
1,7,1.1
1,3
1,1,4.5";
    let rdr = Reader::from_string(data).has_headers(true);
    let res = Matrix::<f64>::read_csv(rdr);
    assert!(res.is_err())
}

#[test]
fn test_write_csv() {
    let mat = matrix![1., 7., 1.1;
                      1., 3., 2.2;
                      1., 1., 4.5];
    let mut wtr = Writer::from_memory();
    mat.write_csv(&mut wtr).unwrap();
    let res = wtr.as_string();
    assert_eq!(res, "1.0,7.0,1.1\n1.0,3.0,2.2\n1.0,1.0,4.5\n");

    // test round-trip
    let rdr = Reader::from_string(res).has_headers(false);
    let res = Matrix::<f64>::read_csv(rdr).unwrap();
    assert_matrix_eq!(res, mat);
}
