//! A hand-rolled separable 2-D DCT-II specialized to a 32×32 input and an
//! 8×8 low-frequency output — the "structure" block of the perceptual
//! embedding. Uses the pHash convention (raw cosine sums, no orthonormal α
//! scaling); the block is L2-normalized downstream, so scaling is irrelevant.

use std::sync::OnceLock;

pub const N: usize = 32;
pub const K: usize = 8;

/// `basis[k][n] = cos(π·(2n+1)·k / (2N))` for k in 0..K, n in 0..N.
fn basis() -> &'static [[f32; N]; K] {
    static BASIS: OnceLock<[[f32; N]; K]> = OnceLock::new();
    BASIS.get_or_init(|| {
        let mut b = [[0f32; N]; K];
        for (k, row) in b.iter_mut().enumerate() {
            for (n, cell) in row.iter_mut().enumerate() {
                let angle =
                    std::f64::consts::PI * ((2 * n + 1) as f64) * (k as f64) / (2.0 * N as f64);
                *cell = angle.cos() as f32;
            }
        }
        b
    })
}

/// 2-D DCT-II of a 32×32 luma matrix (row-major, length 1024); returns the
/// top-left 8×8 coefficients (row-major, length 64).
pub fn dct_top8(luma: &[f32]) -> [f32; K * K] {
    assert_eq!(luma.len(), N * N);
    let b = basis();
    // Separable: first collapse the y (column) axis to the 8 lowest freqs,
    // then the x (row) axis.
    // temp[x][v] = Σ_y f[x][y]·basis[v][y]
    let mut temp = [[0f32; K]; N];
    for (x, trow) in temp.iter_mut().enumerate() {
        let frow = &luma[x * N..x * N + N];
        for (v, tcell) in trow.iter_mut().enumerate() {
            let bv = &b[v];
            let mut acc = 0f32;
            for y in 0..N {
                acc += frow[y] * bv[y];
            }
            *tcell = acc;
        }
    }
    // out[u][v] = Σ_x temp[x][v]·basis[u][x]
    let mut out = [0f32; K * K];
    for u in 0..K {
        let bu = &b[u];
        for v in 0..K {
            let mut acc = 0f32;
            for x in 0..N {
                acc += temp[x][v] * bu[x];
            }
            out[u * K + v] = acc;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference implementation: the direct double sum, no separation.
    fn dct_reference(luma: &[f32], u: usize, v: usize) -> f32 {
        let mut acc = 0f64;
        for x in 0..N {
            for y in 0..N {
                let cu = (std::f64::consts::PI * ((2 * x + 1) as f64) * (u as f64)
                    / (2.0 * N as f64))
                    .cos();
                let cv = (std::f64::consts::PI * ((2 * y + 1) as f64) * (v as f64)
                    / (2.0 * N as f64))
                    .cos();
                acc += (luma[x * N + y] as f64) * cu * cv;
            }
        }
        acc as f32
    }

    #[test]
    fn dct_matches_reference() {
        // A non-trivial deterministic pattern.
        let mut luma = vec![0f32; N * N];
        for x in 0..N {
            for y in 0..N {
                luma[x * N + y] = ((x * 7 + y * 3) % 17) as f32;
            }
        }
        let fast = dct_top8(&luma);
        for u in 0..K {
            for v in 0..K {
                let want = dct_reference(&luma, u, v);
                let got = fast[u * K + v];
                assert!(
                    (want - got).abs() < 1e-2 * (1.0 + want.abs()),
                    "coeff [{u}][{v}]: fast={got} ref={want}"
                );
            }
        }
    }

    #[test]
    fn dc_is_sum_for_constant() {
        // For a constant image, only the DC coefficient is non-zero.
        let luma = vec![5f32; N * N];
        let out = dct_top8(&luma);
        assert!((out[0] - 5.0 * (N * N) as f32).abs() < 1.0);
        for &c in &out[1..] {
            assert!(c.abs() < 1e-2, "AC coeff should be ~0 for constant image");
        }
    }
}
