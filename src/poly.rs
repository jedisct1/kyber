use sp800_185::CShake;
use ::params::{
    N, Q,
    PSIS_BITREV_MONTGOMERY, OMEGAS_MONTGOMER,
    PSIS_INV_MONTGOMERY, OMEGAS_INV_BITREV_MONTGOMERY
};
use ::reduce::{ barrett_reduce, freeze };
use ::cbd::cbd;
use ::ntt::{ bitrev_vector, mul_coefficients, ntt };


pub type Poly = [u16; N];

pub fn poly_compress(poly: &Poly, buf: &mut [u8]) {
    const Q: u32 = ::params::Q as u32;
    let mut t = [0; 8];
    let mut k = 0;

    for i in (0..N).step_by(8) {
        for j in 0..8 {
            t[j] = ((((freeze(poly[i + j]) as u32) << 3) + Q / 2) / Q) & 7;
        }

        buf[k]      = ( t[0]       | (t[1] << 3) | (t[2] << 6))                 as u8;
        buf[k + 1]  = ((t[2] >> 2) | (t[3] << 1) | (t[4] << 4) | (t[5] << 7))   as u8;
        buf[k + 2]  = ((t[5] >> 1) | (t[6] << 2) | (t[7] << 5))                 as u8;
        k += 3;
    }
}

pub fn poly_decompress(poly: &mut Poly, buf: &[u8]) {
    const Q: u16 = ::params::Q as u16;
    let mut a = 0;
    for i in (0..N).step_by(8) {
        poly[i    ] =  ((((buf[a    ] as u16) &  7)                                         * Q) + 4) >> 3;
        poly[i + 1] = (((((buf[a    ] as u16) >> 3) & 7)                                    * Q) + 4) >> 3;
        poly[i + 2] = (((((buf[a    ] as u16) >> 6) | (((buf[a + 1] as u16) << 2) & 4))     * Q) + 4) >> 3;
        poly[i + 3] = (((((buf[a + 1] as u16) >> 1) & 7)                                    * Q) + 4) >> 3;
        poly[i + 4] = (((((buf[a + 1] as u16) >> 4) & 7)                                    * Q) + 4) >> 3;
        poly[i + 5] = (((((buf[a + 1] as u16) >> 7) | (((buf[a + 2] as u16) << 1) & 6))     * Q) + 4) >> 3;
        poly[i + 6] = (((((buf[a + 2] as u16) >> 2) & 7)                                    * Q) + 4) >> 3;
        poly[i + 7] =  ((((buf[a + 2] as u16) >> 5)                                         * Q) + 4) >> 3;
        a += 3;
    }
}


pub fn poly_tobytes(poly: &Poly, buf: &mut [u8]) {
    let mut t = [0; 8];
    for i in 0..(N / 8) {
        for j in 0..8 {
            t[j] = freeze(poly[8 * i + j]);
        }

        buf[13*i   ] =  (t[0]        & 0xff)                    as u8;
        buf[13*i+ 1] = ((t[0] >>  8) | ((t[1] & 0x07) << 5))    as u8;
        buf[13*i+ 2] = ((t[1] >>  3) & 0xff)                    as u8;
        buf[13*i+ 3] = ((t[1] >> 11) | ((t[2] & 0x3f) << 2))    as u8;
        buf[13*i+ 4] = ((t[2] >>  6) | ((t[3] & 0x01) << 7))    as u8;
        buf[13*i+ 5] = ((t[3] >>  1) & 0xff)                    as u8;
        buf[13*i+ 6] = ((t[3] >>  9) | ((t[4] & 0x0f) << 4))    as u8;
        buf[13*i+ 7] = ((t[4] >>  4) & 0xff)                    as u8;
        buf[13*i+ 8] = ((t[4] >> 12) | ((t[5] & 0x7f) << 1))    as u8;
        buf[13*i+ 9] = ((t[5] >>  7) | ((t[6] & 0x03) << 6))    as u8;
        buf[13*i+10] = ((t[6] >>  2) & 0xff)                    as u8;
        buf[13*i+11] = ((t[6] >> 10) | ((t[7] & 0x1f) << 3))    as u8;
        buf[13*i+12] =  (t[7] >>  5)                            as u8;
    }
}

pub fn poly_frombytes(poly: &mut Poly, buf: &[u8]) {
    for i in 0..(N / 8) {
        poly[8*i  ] =  (buf[13*i   ] as u16)       | ((buf[13*i+ 1] as u16 & 0x1f) << 8);
        poly[8*i+1] = ((buf[13*i+ 1] as u16) >> 5) | ((buf[13*i+ 2] as u16       ) << 3) | (((buf[13*i+ 3] as u16) & 0x03) << 11);
        poly[8*i+2] = ((buf[13*i+ 3] as u16) >> 2) | ((buf[13*i+ 4] as u16 & 0x7f) << 6);
        poly[8*i+3] = ((buf[13*i+ 4] as u16) >> 7) | ((buf[13*i+ 5] as u16       ) << 1) | (((buf[13*i+ 6] as u16) & 0x0f) <<  9);
        poly[8*i+4] = ((buf[13*i+ 6] as u16) >> 4) | ((buf[13*i+ 7] as u16       ) << 4) | (((buf[13*i+ 8] as u16) & 0x01) << 12);
        poly[8*i+5] = ((buf[13*i+ 8] as u16) >> 1) | ((buf[13*i+ 9] as u16 & 0x3f) << 7);
        poly[8*i+6] = ((buf[13*i+ 9] as u16) >> 6) | ((buf[13*i+10] as u16       ) << 2) | (((buf[13*i+11] as u16) & 0x07) << 10);
        poly[8*i+7] = ((buf[13*i+11] as u16) >> 3) | ((buf[13*i+12] as u16       ) << 5);
    }
}

pub fn poly_getnoise(poly: &mut Poly, seed: &[u8], nonce: u8) {
    let mut buf = [0; N];

    let mut cshake = CShake::new_cshake128(&[], &[nonce, 0x00]);
    cshake.update(seed);
    cshake.finalize(&mut buf);

    cbd(poly, &buf);
}

pub fn poly_ntt(poly: &mut Poly) {
    mul_coefficients(poly, &PSIS_BITREV_MONTGOMERY);
    ntt(poly, &OMEGAS_MONTGOMER);
}

pub fn poly_invntt(poly: &mut Poly) {
    bitrev_vector(poly);
    ntt(poly, &OMEGAS_INV_BITREV_MONTGOMERY);
    mul_coefficients(poly, &PSIS_INV_MONTGOMERY);
}

pub fn poly_add(r: &mut Poly, a: &Poly, b: &Poly) {
    for i in 0..N {
        r[i] = barrett_reduce(a[i] + b[i]);
    }
}

pub fn poly_sub(r: &mut Poly, a: &Poly, b: &Poly) {
    for i in 0..N {
        r[i] = barrett_reduce(a[i] + 3 * Q as u16 - b[i]);
    }
}
