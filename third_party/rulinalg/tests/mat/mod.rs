use rulinalg::matrix::{BaseMatrix};

#[test]
fn test_solve() {
    let a = matrix![-4.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0;
                    1.0, -4.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0;
                    0.0, 1.0, -4.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0;
                    1.0, 0.0, 0.0, -4.0, 1.0, 0.0, 1.0, 0.0, 0.0;
                    0.0, 1.0, 0.0, 1.0, -4.0, 1.0, 0.0, 1.0, 0.0;
                    0.0, 0.0, 1.0, 0.0, 1.0, -4.0, 0.0, 0.0, 1.0;
                    0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -4.0, 1.0, 0.0;
                    0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, -4.0, 1.0;
                    0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, -4.0];

    let b = vector![-100.0, 0.0, 0.0, -100.0, 0.0, 0.0, -100.0, 0.0, 0.0];

    let c = a.solve(b).unwrap();
    let true_solution = vector![42.85714286, 18.75, 7.14285714, 52.67857143,
                                25.0, 9.82142857, 42.85714286, 18.75, 7.14285714];

    // Note: the "true_solution" given here has way too few
    // significant digits, and since I can't be bothered to enter
    // it all into e.g. NumPy, I'm leaving a lower absolute
    // tolerance in place.
    assert_vector_eq!(c, true_solution, comp = abs, tol = 1e-8);
}

#[test]
fn test_l_triangular_solve_errs() {
    let a = matrix![0.0];
    assert!(a.solve_l_triangular(vector![1.0]).is_err());
}

#[test]
fn test_u_triangular_solve_errs() {
    let a = matrix![0.0];
    assert!(a.solve_u_triangular(vector![1.0]).is_err());
}

#[allow(deprecated)]
#[test]
fn matrix_lup_decomp() {
    let a = matrix![1., 3., 5.;
                    2., 4., 7.;
                    1., 1., 0.];

    let (l, u, p) = a.lup_decomp().expect("Matrix SHOULD be able to be decomposed...");

    let l_true = vec![1., 0., 0., 0.5, 1., 0., 0.5, -1., 1.];
    let u_true = vec![2., 4., 7., 0., 1., 1.5, 0., 0., -2.];
    let p_true = vec![0., 1., 0., 1., 0., 0., 0., 0., 1.];

    assert_eq!(*p.data(), p_true);
    assert_eq!(*l.data(), l_true);
    assert_eq!(*u.data(), u_true);

    let b = matrix![1., 2., 3., 4., 5.;
                    3., 0., 4., 5., 6.;
                    2., 1., 2., 3., 4.;
                    0., 0., 0., 6., 5.;
                    0., 0., 0., 5., 6.];

    let (l, u, p) = b.clone().lup_decomp().expect("Matrix SHOULD be able to be decomposed...");
    let k = p.transpose() * l * u;

    for i in 0..25 {
        assert_eq!(b.data()[i], k.data()[i]);
    }

    let c = matrix![-4.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0;
                    1.0, -4.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0;
                    0.0, 1.0, -4.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0;
                    1.0, 0.0, 0.0, -4.0, 1.0, 0.0, 1.0, 0.0, 0.0;
                    0.0, 1.0, 0.0, 1.0, -4.0, 1.0, 0.0, 1.0, 0.0;
                    0.0, 0.0, 1.0, 0.0, 1.0, -4.0, 0.0, 0.0, 1.0;
                    0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -4.0, 1.0, 0.0;
                    0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, -4.0, 1.0;
                    0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, -4.0];

    assert!(c.lup_decomp().is_ok());

    let d = matrix![1.0, 1.0, 0.0, 0.0;
                    0.0, 0.0, 1.0, 0.0;
                    -1.0, 0.0, 0.0, 0.0;
                    0.0, 0.0, 0.0, 1.0];

    assert!(d.lup_decomp().is_ok());
}

#[test]
fn matrix_partial_piv_lu() {
    use rulinalg::matrix::decomposition::{LUP, PartialPivLu};
    use rulinalg::matrix::decomposition::Decomposition;
    // This is a port of the test for the old lup_decomp
    // function, using the new PartialPivLu struct.

    {
        // Note: this test only works with a _specific_
        // implementation of LU decomposition, because
        // an LUP decomposition is not in general unique,
        // so one cannot test against expected L, U and P
        // matrices unless one knows exactly which ones the
        // algorithm works.
        let a = matrix![1., 3., 5.;
                        2., 4., 7.;
                        1., 1., 0.];

        let LUP { l, u, p } = PartialPivLu::decompose(a)
                                        .expect("Matrix is well-conditioned")
                                        .unpack();

        let l_true = matrix![1.0,  0.0,  0.0;
                             0.5,  1.0,  0.0;
                             0.5, -1.0,  1.0];
        let u_true = matrix![2.0,  4.0,  7.0;
                             0.0,  1.0,  1.5;
                             0.0,  0.0, -2.0];
        let p_true = matrix![0.0, 1.0, 0.0;
                            1.0, 0.0, 0.0;
                            0.0, 0.0, 1.0];

        assert_matrix_eq!(l, l_true, comp = float);
        assert_matrix_eq!(u, u_true, comp = float);
        assert_matrix_eq!(p.as_matrix(), p_true, comp = float);
    }

    {
        let b = matrix![1., 2., 3., 4., 5.;
                        3., 0., 4., 5., 6.;
                        2., 1., 2., 3., 4.;
                        0., 0., 0., 6., 5.;
                        0., 0., 0., 5., 6.];

        let LUP { l, u, p } = PartialPivLu::decompose(b.clone())
                         .expect("Matrix is well-conditioned.")
                         .unpack();
        let k = p.inverse() * l * u;

        assert_matrix_eq!(k, b, comp = float);
    }

    {
        let c = matrix![-4.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0;
                        1.0, -4.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0;
                        0.0, 1.0, -4.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0;
                        1.0, 0.0, 0.0, -4.0, 1.0, 0.0, 1.0, 0.0, 0.0;
                        0.0, 1.0, 0.0, 1.0, -4.0, 1.0, 0.0, 1.0, 0.0;
                        0.0, 0.0, 1.0, 0.0, 1.0, -4.0, 0.0, 0.0, 1.0;
                        0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -4.0, 1.0, 0.0;
                        0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, -4.0, 1.0;
                        0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, -4.0];

        let LUP { l, u, p } = PartialPivLu::decompose(c.clone())
                                .unwrap()
                                .unpack();
        let c_reconstructed = p.inverse() * l * u;
        assert_matrix_eq!(c_reconstructed, c, comp = float);
    }

    {
        let d = matrix![1.0, 1.0, 0.0, 0.0;
                        0.0, 0.0, 1.0, 0.0;
                        -1.0, 0.0, 0.0, 0.0;
                        0.0, 0.0, 0.0, 1.0];
        let LUP { l, u, p } = PartialPivLu::decompose(d.clone())
                                .unwrap()
                                .unpack();
        let d_reconstructed = p.inverse() * l * u;
        assert_matrix_eq!(d_reconstructed, d, comp = float);
    }
}

#[test]
#[allow(deprecated)]
fn cholesky() {
    let a = matrix![25., 15., -5.;
                    15., 18., 0.;
                    -5., 0., 11.];

    let l = a.cholesky();

    assert!(l.is_ok());

    assert_eq!(*l.unwrap().data(), vec![5., 0., 0., 3., 3., 0., -1., 1., 3.]);
}

#[test]
#[allow(deprecated)]
fn qr() {
    let a = matrix![12., -51., 4.;
                    6., 167., -68.;
                    -4., 24., -41.];

    let (q, r) = a.qr_decomp().unwrap();

    let true_q = matrix![-0.857143, 0.394286, 0.331429;
                         -0.428571, -0.902857, -0.034286;
                         0.285715, -0.171429, 0.942857];
    let true_r = matrix![-14., -21., 14.;
                         0., -175., 70.;
                         0., 0., -35.];

    assert_matrix_eq!(q, true_q, comp = abs, tol = 1e-6);
    assert_matrix_eq!(r, true_r, comp = abs, tol = 1e-6);
}
