import rotor;

struct Transform
{
    float s;
    float e01;
    float e02;
    float e03;
    float e04;
    float e12;
    float e13;
    float e14;
    float e23;
    float e24;
    float e34;
    float e0123;
    float e0124;
    float e0134;
    float e0234;
    float e1234;

    Transform inverse()
    {
        var result : Transform;
        result.s = this.s;
        result.e01 = -this.e01;
        result.e02 = -this.e02;
        result.e03 = -this.e03;
        result.e04 = -this.e04;
        result.e12 = -this.e12;
        result.e13 = -this.e13;
        result.e14 = -this.e14;
        result.e23 = -this.e23;
        result.e24 = -this.e24;
        result.e34 = -this.e34;
        result.e0123 = this.e0123;
        result.e0124 = this.e0124;
        result.e0134 = this.e0134;
        result.e0234 = this.e0234;
        result.e1234 = this.e1234;
        return result;
    }

    Transform then(Transform other)
    {
        let a1 = this.s;
        let b1 = this.e01;
        let c1 = this.e02;
        let d1 = this.e03;
        let f1 = this.e04;
        let g1 = this.e12;
        let h1 = this.e13;
        let i1 = this.e14;
        let j1 = this.e23;
        let k1 = this.e24;
        let l1 = this.e34;
        let m1 = this.e0123;
        let n1 = this.e0124;
        let o1 = this.e0134;
        let p1 = this.e0234;
        let q1 = this.e1234;
        let a2 = other.s;
        let b2 = other.e01;
        let c2 = other.e02;
        let d2 = other.e03;
        let f2 = other.e04;
        let g2 = other.e12;
        let h2 = other.e13;
        let i2 = other.e14;
        let j2 = other.e23;
        let k2 = other.e24;
        let l2 = other.e34;
        let m2 = other.e0123;
        let n2 = other.e0124;
        let o2 = other.e0134;
        let p2 = other.e0234;
        let q2 = other.e1234;
        var result : Transform;
        result.s = -g1 * g2 + -h1 * h2 + -i1 * i2 + -j1 * j2 + -k1 * k2 + -l1 * l2 + a1 * a2 + q1 * q2,
        result.e01 = -c1 * g2 + -d1 * h2 + -f1 * i2 + -j1 * m2 + -j2 * m1 + -k1 * n2 + -k2 * n1 + -l1 * o2 + -l2 * o1 + -p2 * q1 + a1 * b2 + a2 * b1 + c2 * g1 + d2 * h1 + f2 * i1 + p1 * q2;
        result.e02 = -b2 * g1 + -d1 * j2 + -f1 * k2 + -l1 * p2 + -l2 * p1 + -o1 * q2 + a1 * c2 + a2 * c1 + b1 * g2 + d2 * j1 + f2 * k1 + h1 * m2 + h2 * m1 + i1 * n2 + i2 * n1 + o2 * q1;
        result.e03 = -b2 * h1 + -c2 * j1 + -f1 * l2 + -g1 * m2 + -g2 * m1 + -n2 * q1 + a1 * d2 + a2 * d1 + b1 * h2 + c1 * j2 + f2 * l1 + i1 * o2 + i2 * o1 + k1 * p2 + k2 * p1 + n1 * q2;
        result.e04 = -b2 * i1 + -c2 * k1 + -d2 * l1 + -g1 * n2 + -g2 * n1 + -h1 * o2 + -h2 * o1 + -j1 * p2 + -j2 * p1 + -m1 * q2 + a1 * f2 + a2 * f1 + b1 * i2 + c1 * k2 + d1 * l2 + m2 * q1;
        result.e12 = -h1 * j2 + -i1 * k2 + -l1 * q2 + -l2 * q1 + a1 * g2 + a2 * g1 + h2 * j1 + i2 * k1;
        result.e13 = -g2 * j1 + -i1 * l2 + a1 * h2 + a2 * h1 + g1 * j2 + i2 * l1 + k1 * q2 + k2 * q1;
        result.e14 = -g2 * k1 + -h2 * l1 + -j1 * q2 + -j2 * q1 + a1 * i2 + a2 * i1 + g1 * k2 + h1 * l2;
        result.e23 = -g1 * h2 + -i1 * q2 + -i2 * q1 + -k1 * l2 + a1 * j2 + a2 * j1 + g2 * h1 + k2 * l1;
        result.e24 = -g1 * i2 + -j2 * l1 + a1 * k2 + a2 * k1 + g2 * i1 + h1 * q2 + h2 * q1 + j1 * l2;
        result.e34 = -g1 * q2 + -g2 * q1 + -h1 * i2 + -j1 * k2 + a1 * l2 + a2 * l1 + h2 * i1 + j2 * k1;
        result.e0123 = -c1 * h2 + -c2 * h1 + -f1 * q2 + -i2 * p1 + -k1 * o2 + -l2 * n1 + a1 * m2 + a2 * m1 + b1 * j2 + b2 * j1 + d1 * g2 + d2 * g1 + f2 * q1 + i1 * p2 + k2 * o1 + l1 * n2;
        result.e0124 = -c1 * i2 + -c2 * i1 + -d2 * q1 + -h1 * p2 + -j2 * o1 + -l1 * m2 + a1 * n2 + a2 * n1 + b1 * k2 + b2 * k1 + d1 * q2 + f1 * g2 + f2 * g1 + h2 * p1 + j1 * o2 + l2 * m1;
        result.e0134 = -c1 * q2 + -d1 * i2 + -d2 * i1 + -g2 * p1 + -j1 * n2 + -k2 * m1 + a1 * o2 + a2 * o1 + b1 * l2 + b2 * l1 + c2 * q1 + f1 * h2 + f2 * h1 + g1 * p2 + j2 * n1 + k1 * m2;
        result.e0234 = -b2 * q1 + -d1 * k2 + -d2 * k1 + -g1 * o2 + -h2 * n1 + -i1 * m2 + a1 * p2 + a2 * p1 + b1 * q2 + c1 * l2 + c2 * l1 + f1 * j2 + f2 * j1 + g2 * o1 + h1 * n2 + i2 * m1;
        result.e1234 = -h1 * k2 + -h2 * k1 + a1 * q2 + a2 * q1 + g1 * l2 + g2 * l1 + i1 * j2 + i2 * j1;
        return result;
    }

    float4 transform(float4 point)
    {
        let a = this.s;
        let b = this.e01;
        let c = this.e02;
        let d = this.e03;
        let f = this.e04;
        let g = this.e12;
        let h = this.e13;
        let i = this.e14;
        let j = this.e23;
        let k = this.e24;
        let l = this.e34;
        let m = this.e0123;
        let n = this.e0124;
        let o = this.e0134;
        let p = this.e0234;
        let q = this.e1234;
        let p3 = point.x;
        let p2 = point.y;
        let p1 = point.z;
        let p0 = point.w;
        let ap2 = a * p2;
        let gp3 = g * p3;
        let jp1 = j * p1;
        let kp0 = k * p0;
        let ap3 = a * p3;
        let gp2 = g * p2;
        let hp1 = h * p1;
        let ip0 = i * p0;
        let ap1 = a * p1;
        let lp0 = l * p0;
        let hp3 = h * p3;
        let jp2 = j * p2;
        let ap0 = a * p0;
        let lp1 = l * p1;
        let ip3 = i * p3;
        let kp2 = k * p2;
        let s0 = c + jp1 - ap2 - gp3 - kp0;
        let s1 = ap3 + b + hp1 - gp2 - ip0;
        let s2 = ap1 + d + jp2 - lp0 - hp3;
        let s3 = f + kp2 - ap0 - lp1 - ip3;
        let new_point = float4(
            p0 + 2.0 * (q * (m + g * p1 + h * p2 + j * p3 - q * p0) + k * s0 + i * s1 + l * s2 - a * f - n * g - o * h - p * j),
            p1 + 2.0 * (a * d + m * g + q * (n + i * p2 + k * p3 - q * p1 - g * p0) + l * s3 - o * i - p * k - j * s0 - h * s1),
            p2 + 2.0 * (m * h + n * i + q * (l * p3 + o - q * p2 - h * p0 - i * p1) + g * s1 - a * c - l * p - k * s3 - j * s2),
            p3 + 2.0 * (a * b + l * o + m * j + n * k + q * (p - l * p2 - q * p3 - j * p0 - k * p1) + i * s3 + h * s2 + g * s0));
        return new_point.wzyx;
    }

    Rotor rotor_part()
    {
        var result : Rotor;
        result.s = this.s;
        result.e12 = this.e12;
        result.e13 = this.e13;
        result.e14 = this.e14;
        result.e23 = this.e23;
        result.e24 = this.e24;
        result.e34 = this.e34;
        result.e1234 = this.e1234;
        return result;
    }
}
