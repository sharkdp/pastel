use super::Lab;
use std::f64;

// The code below is adapted from https://github.com/elliotekj/DeltaE
//
// Original license:
//
// The MIT License (MIT)
//
// Copyright (c) 2017 Elliot Jackson
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

pub fn cie76(c1: &Lab, c2: &Lab) -> f64 {
    ((c1.l - c2.l).powi(2) + (c1.a - c2.a).powi(2) + (c1.b - c2.b).powi(2)).sqrt()
}

pub fn ciede2000(color1: &Lab, color2: &Lab) -> f64 {
    let ksub_l = 1.0;
    let ksub_c = 1.0;
    let ksub_h = 1.0;

    let delta_l_prime = color2.l - color1.l;

    let l_bar = (color1.l + color2.l) / 2.0;

    let c1 = (color1.a.powi(2) + color1.b.powi(2)).sqrt();
    let c2 = (color2.a.powi(2) + color2.b.powi(2)).sqrt();

    let c_bar = (c1 + c2) / 2.0;

    let a_prime_1 = color1.a
        + (color1.a / 2.0) * (1.0 - (c_bar.powi(7) / (c_bar.powi(7) + 25f64.powi(7))).sqrt());
    let a_prime_2 = color2.a
        + (color2.a / 2.0) * (1.0 - (c_bar.powi(7) / (c_bar.powi(7) + 25f64.powi(7))).sqrt());

    let c_prime_1 = (a_prime_1.powi(2) + color1.b.powi(2)).sqrt();
    let c_prime_2 = (a_prime_2.powi(2) + color2.b.powi(2)).sqrt();

    let c_bar_prime = (c_prime_1 + c_prime_2) / 2.0;

    let delta_c_prime = c_prime_2 - c_prime_1;

    let s_sub_l = 1.0 + ((0.015 * (l_bar - 50.0).powi(2)) / (20.0 + (l_bar - 50.0).powi(2)).sqrt());

    let s_sub_c = 1.0 + 0.045 * c_bar_prime;

    let h_prime_1 = get_h_prime_fn(color1.b, a_prime_1);
    let h_prime_2 = get_h_prime_fn(color2.b, a_prime_2);

    let delta_h_prime = get_delta_h_prime(c1, c2, h_prime_1, h_prime_2);

    let delta_upcase_h_prime =
        2.0 * (c_prime_1 * c_prime_2).sqrt() * (degrees_to_radians(delta_h_prime) / 2.0).sin();

    let upcase_h_bar_prime = get_upcase_h_bar_prime(h_prime_1, h_prime_2);

    let upcase_t = get_upcase_t(upcase_h_bar_prime);

    let s_sub_upcase_h = 1.0 + 0.015 * c_bar_prime * upcase_t;

    let r_sub_t = get_r_sub_t(c_bar_prime, upcase_h_bar_prime);

    let lightness: f64 = delta_l_prime / (ksub_l * s_sub_l);

    let chroma: f64 = delta_c_prime / (ksub_c * s_sub_c);

    let hue: f64 = delta_upcase_h_prime / (ksub_h * s_sub_upcase_h);

    (lightness.powi(2) + chroma.powi(2) + hue.powi(2) + r_sub_t * chroma * hue).sqrt()
}

fn get_h_prime_fn(x: f64, y: f64) -> f64 {
    if x == 0.0 && y == 0.0 {
        return 0.0;
    }

    let mut hue_angle = radians_to_degrees(x.atan2(y));

    if hue_angle < 0.0 {
        hue_angle += 360.0;
    }

    hue_angle
}

fn get_delta_h_prime(c1: f64, c2: f64, h_prime_1: f64, h_prime_2: f64) -> f64 {
    if 0.0 == c1 || 0.0 == c2 {
        return 0.0;
    }

    if (h_prime_1 - h_prime_2).abs() <= 180.0 {
        return h_prime_2 - h_prime_1;
    }

    if h_prime_2 <= h_prime_1 {
        h_prime_2 - h_prime_1 + 360.0
    } else {
        h_prime_2 - h_prime_1 - 360.0
    }
}

fn get_upcase_h_bar_prime(h_prime_1: f64, h_prime_2: f64) -> f64 {
    if (h_prime_1 - h_prime_2).abs() > 180.0 {
        return (h_prime_1 + h_prime_2 + 360.0) / 2.0;
    }

    (h_prime_1 + h_prime_2) / 2.0
}

fn get_upcase_t(upcase_h_bar_prime: f64) -> f64 {
    1.0 - 0.17 * (degrees_to_radians(upcase_h_bar_prime - 30.0)).cos()
        + 0.24 * (degrees_to_radians(2.0 * upcase_h_bar_prime)).cos()
        + 0.32 * (degrees_to_radians(3.0 * upcase_h_bar_prime + 6.0)).cos()
        - 0.20 * (degrees_to_radians(4.0 * upcase_h_bar_prime - 63.0)).cos()
}

fn get_r_sub_t(c_bar_prime: f64, upcase_h_bar_prime: f64) -> f64 {
    -2.0 * (c_bar_prime.powi(7) / (c_bar_prime.powi(7) + 25f64.powi(7))).sqrt()
        * (degrees_to_radians(60.0 * (-(((upcase_h_bar_prime - 275.0) / 25.0).powi(2))).exp()))
            .sin()
}

fn radians_to_degrees(radians: f64) -> f64 {
    radians * (180.0 / f64::consts::PI)
}

fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * (f64::consts::PI / 180.0)
}

#[cfg(test)]
mod tests {
    use super::{ciede2000, Lab};

    fn round(val: f64) -> f64 {
        let rounded = val * 10000_f64;
        rounded.round() / 10000_f64
    }

    fn assert_delta_e(expected: f64, lab1: &[f64; 3], lab2: &[f64; 3]) {
        let color1 = Lab {
            l: lab1[0],
            a: lab1[1],
            b: lab1[2],
            alpha: 1.0,
        };

        let color2 = Lab {
            l: lab2[0],
            a: lab2[1],
            b: lab2[2],
            alpha: 1.0,
        };

        assert_eq!(round(ciede2000(&color1, &color2)), expected);
    }

    // Tests taken from Table 1: "CIEDE2000 total color difference test data" of
    // "The CIEDE2000 Color-Difference Formula: Implementation Notes,
    // Supplementary Test Data, and Mathematical Observations" by Gaurav Sharma,
    // Wencheng Wu and Edul N. Dalal.
    //
    // http://www.ece.rochester.edu/~gsharma/papers/CIEDE2000CRNAFeb05.pdf

    #[test]
    fn tests() {
        assert_delta_e(0.0, &[0.0, 0.0, 0.0], &[0.0, 0.0, 0.0]);
        assert_delta_e(0.0, &[99.5, 0.005, -0.010], &[99.5, 0.005, -0.010]);
        assert_delta_e(100.0, &[100.0, 0.005, -0.010], &[0.0, 0.0, 0.0]);
        assert_delta_e(
            2.0425,
            &[50.0000, 2.6772, -79.7751],
            &[50.0000, 0.0000, -82.7485],
        );
        assert_delta_e(
            2.8615,
            &[50.0000, 3.1571, -77.2803],
            &[50.0000, 0.0000, -82.7485],
        );
        assert_delta_e(
            3.4412,
            &[50.0000, 2.8361, -74.0200],
            &[50.0000, 0.0000, -82.7485],
        );
        assert_delta_e(
            1.0000,
            &[50.0000, -1.3802, -84.2814],
            &[50.0000, 0.0000, -82.7485],
        );
        assert_delta_e(
            1.0000,
            &[50.0000, -1.1848, -84.8006],
            &[50.0000, 0.0000, -82.7485],
        );
        assert_delta_e(
            1.0000,
            &[50.0000, -0.9009, -85.5211],
            &[50.0000, 0.0000, -82.7485],
        );
        assert_delta_e(
            2.3669,
            &[50.0000, 0.0000, 0.0000],
            &[50.0000, -1.0000, 2.0000],
        );
        assert_delta_e(
            2.3669,
            &[50.0000, -1.0000, 2.0000],
            &[50.0000, 0.0000, 0.0000],
        );
        assert_delta_e(
            7.1792,
            &[50.0000, 2.4900, -0.0010],
            &[50.0000, -2.4900, 0.0009],
        );
        assert_delta_e(
            7.1792,
            &[50.0000, 2.4900, -0.0010],
            &[50.0000, -2.4900, 0.0010],
        );
        assert_delta_e(
            7.2195,
            &[50.0000, 2.4900, -0.0010],
            &[50.0000, -2.4900, 0.0011],
        );
        assert_delta_e(
            7.2195,
            &[50.0000, 2.4900, -0.0010],
            &[50.0000, -2.4900, 0.0012],
        );
        assert_delta_e(
            4.8045,
            &[50.0000, -0.0010, 2.4900],
            &[50.0000, 0.0009, -2.4900],
        );
        assert_delta_e(
            4.7461,
            &[50.0000, -0.0010, 2.4900],
            &[50.0000, 0.0011, -2.4900],
        );
        assert_delta_e(
            4.3065,
            &[50.0000, 2.5000, 0.0000],
            &[50.0000, 0.0000, -2.5000],
        );
        assert_delta_e(
            27.1492,
            &[50.0000, 2.5000, 0.0000],
            &[73.0000, 25.0000, -18.0000],
        );
        assert_delta_e(
            22.8977,
            &[50.0000, 2.5000, 0.0000],
            &[61.0000, -5.0000, 29.0000],
        );
        assert_delta_e(
            31.9030,
            &[50.0000, 2.5000, 0.0000],
            &[56.0000, -27.0000, -3.0000],
        );
        assert_delta_e(
            19.4535,
            &[50.0000, 2.5000, 0.0000],
            &[58.0000, 24.0000, 15.0000],
        );
        assert_delta_e(
            1.0000,
            &[50.0000, 2.5000, 0.0000],
            &[50.0000, 3.1736, 0.5854],
        );
        assert_delta_e(
            1.0000,
            &[50.0000, 2.5000, 0.0000],
            &[50.0000, 3.2972, 0.0000],
        );
        assert_delta_e(
            1.0000,
            &[50.0000, 2.5000, 0.0000],
            &[50.0000, 1.8634, 0.5757],
        );
        assert_delta_e(
            1.0000,
            &[50.0000, 2.5000, 0.0000],
            &[50.0000, 3.2592, 0.3350],
        );
        assert_delta_e(
            1.2644,
            &[60.2574, -34.0099, 36.2677],
            &[60.4626, -34.1751, 39.4387],
        );
        assert_delta_e(
            1.2630,
            &[63.0109, -31.0961, -5.8663],
            &[62.8187, -29.7946, -4.0864],
        );
        assert_delta_e(
            1.8731,
            &[61.2901, 3.7196, -5.3901],
            &[61.4292, 2.2480, -4.9620],
        );
        assert_delta_e(
            1.8645,
            &[35.0831, -44.1164, 3.7933],
            &[35.0232, -40.0716, 1.5901],
        );
        assert_delta_e(
            2.0373,
            &[22.7233, 20.0904, -46.6940],
            &[23.0331, 14.9730, -42.5619],
        );
        assert_delta_e(
            1.4146,
            &[36.4612, 47.8580, 18.3852],
            &[36.2715, 50.5065, 21.2231],
        );
        assert_delta_e(
            1.4441,
            &[90.8027, -2.0831, 1.4410],
            &[91.1528, -1.6435, 0.0447],
        );
        assert_delta_e(
            1.5381,
            &[90.9257, -0.5406, -0.9208],
            &[88.6381, -0.8985, -0.7239],
        );
        assert_delta_e(
            0.6377,
            &[6.7747, -0.2908, -2.4247],
            &[5.8714, -0.0985, -2.2286],
        );
        assert_delta_e(
            0.9082,
            &[2.0776, 0.0795, -1.1350],
            &[0.9033, -0.0636, -0.5514],
        );
    }
}
